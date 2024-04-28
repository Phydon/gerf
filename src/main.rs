use clap::{Arg, ArgAction, ArgMatches, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::{error, info, warn};
use owo_colors::colored::*;
use rand::prelude::*;
use rayon::prelude::*;

use std::{
    fs, io,
    path::{Path, PathBuf},
    process,
};

// maximum filesize possible; restricted for safety reason; may change in the future
const MAXSIZE: u64 = 5 * 1024_u64.pow(3); // 5 GB

// warn user when this filesize gets exceeded
const WARNSIZE: u32 = 100 * 1024_u32.pow(2); // 100 MB
const KB: u16 = 1024;
const MB: u32 = 1024_u32.pow(2);
const GB: u32 = 1024_u32.pow(3);

const NUMS: [&'static str; 12] = [" ", "\n", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
const LOREM: [&'static str; 12] = [
    // fill file with this random 'lorem ipsum' like content
    " ",
    "\n",
    "et",
    "est",
    "elit",
    "wasd",
    " ",
    "dolor",
    "labore",
    "eiusmod",
    "aliquaer",
    "adipisici",
];

fn main() {
    // handle Ctrl+C
    ctrlc::set_handler(move || {
        println!("{}", "Received Ctrl-C!".italic(),);
        process::exit(0)
    })
    .expect("Error setting Ctrl-C handler");

    // get config dir
    let config_dir = check_create_config_dir().unwrap_or_else(|err| {
        error!("Unable to find or create a config directory: {err}");
        process::exit(1);
    });

    // initialize the logger
    let _logger = Logger::try_with_str("info") // log warn and error
        .unwrap()
        .format_for_files(detailed_format) // use timestamp for every log
        .log_to_file(
            FileSpec::default()
                .directory(&config_dir)
                .suppress_timestamp(),
        ) // change directory for logs, no timestamps in the filename
        .append() // use only one logfile
        .duplicate_to_stderr(Duplicate::Info) // print infos, warnings and errors also to the console
        .start()
        .unwrap();

    // handle arguments
    let matches = gerf().get_matches();
    let exceed_flag = matches.get_flag("exceed");
    let nums_flag = matches.get_flag("nums");
    let override_flag = matches.get_flag("override");
    let words_flag = matches.get_flag("words");

    if let Some(_) = matches.subcommand_matches("log") {
        if let Ok(logs) = show_log_file(&config_dir) {
            println!("{}", "Available logs:".bold().yellow());
            println!("{}", logs);
        } else {
            error!("Unable to read logs");
            process::exit(1);
        }
    } else if let Some(_) = matches.subcommand_matches("examples") {
        examples();
    } else {
        if let Some(s) = matches.get_one::<String>("size") {
            let filesize: u64;
            match s.parse() {
                Ok(s) => filesize = s,
                Err(err) => {
                    warn!("Expected an integer as filesize: {err}");
                    process::exit(0);
                }
            }

            // convert size input if any unit flag is set
            // byte is the default unit
            let size = Size::from(filesize, &matches).convert();

            // maximum filesize possible -> for safety reason
            if size > MAXSIZE {
                warn!(
                    "{} Bytes exceed the allowed maximum size of {} Bytes",
                    size, MAXSIZE
                );
                process::exit(0);
            }

            // make sure the user doesn't accidentally produces hugh files
            if size > WARNSIZE as u64 {
                if !exceed_flag {
                    warn!(
                        "{} Bytes exceed the default restriction size of {} Bytes",
                        size, WARNSIZE
                    );
                    info!(
                            "Use the [ -e ] or [ --exceed ] flag to exceed the default restriction size"
                    );
                    process::exit(0);
                } else {
                    // let user confirm before producing big files
                    let_user_confirm();
                }
            }

            // get custom path from user, if none -> use default path
            let mut path = PathBuf::new();
            if let Some(p) = matches.get_one::<String>("path") {
                path.push(p);
            }

            // force user to use --override flag before overriding existing files
            if path.exists() && !override_flag {
                warn!("The file '{}' already exists!", path.display());
                info!("Use the [ -o ] or [ --override ] flag to override the existing file");
                process::exit(0);
            }

            // create file of given size and fill with random content
            if words_flag {
                Content::new()
                    .genrand_content(size)
                    .shrink_to_size(size)
                    .populate_file(path);
            } else if nums_flag {
                Content::new()
                    .genrand_num(size)
                    .shrink_to_size(size)
                    .populate_file(path);
            } else {
                Content::new()
                    .genrand_content(size)
                    .shrink_to_size(size)
                    .populate_file(path);
            };
        } else {
            let _ = gerf().print_help();
            process::exit(0);
        }
    }
}

#[derive(Debug)]
enum Unit {
    Byte,
    Kilobyte,
    Megabyte,
    Gigabyte,
}

#[derive(Debug)]
struct Size {
    size: u64,
    unit: Unit,
}

impl Size {
    fn from(size: u64, matches: &ArgMatches) -> Self {
        // construct unit based on the given unit flag
        let unit = if matches.get_flag("kb") {
            Unit::Kilobyte
        } else if matches.get_flag("mb") {
            Unit::Megabyte
        } else if matches.get_flag("gb") {
            Unit::Gigabyte
        } else {
            // default
            Unit::Byte
        };

        Size { size, unit }
    }

    fn convert(&self) -> u64 {
        // convert the given size (based on the given unit) to bytes
        match self.unit {
            Unit::Kilobyte => return (self.size as f64 * KB as f64) as u64,
            Unit::Megabyte => return (self.size as f64 * MB as f64) as u64,
            Unit::Gigabyte => return (self.size as f64 * GB as f64) as u64,
            _ => return self.size,
        }
    }
}

#[derive(Debug, Clone)]
struct Content {
    lines: Vec<&'static str>,
}

impl Content {
    fn new() -> Self {
        Content { lines: Vec::new() }
    }

    #[cfg(test)]
    fn from(vec: Vec<&'static str>) -> Self {
        Content { lines: vec }
    }

    // TODO generate different "random" content
    // TODO generate content with only numbers
    // TODO generate content with only words
    // TODO generate alphanumeric content
    fn genrand_content(&mut self, size: u64) -> &mut Self {
        // generate random content to fill the file
        let mut rng = thread_rng();
        let mut length: u64 = 0;

        for _ in 1..size {
            length += self.lines.last().unwrap_or(&"").len() as u64;

            if length >= size {
                break;
            }

            let i: u8 = rng.gen_range(0..=(LOREM.len() - 1) as u8);
            self.lines.push(LOREM[i as usize]);
        }

        self
    }

    fn genrand_num(&mut self, size: u64) -> &mut Self {
        // generate random numbers as file content
        let mut rng = thread_rng();
        let mut length: u64 = 0;

        for _ in 1..size {
            length += self.lines.last().unwrap_or(&"").len() as u64;

            if length >= size {
                break;
            }

            let i: u64 = rng.gen_range(0..10);
            self.lines.push(NUMS[i as usize]);
        }

        self
    }

    fn shrink_to_size(&mut self, size: u64) -> Self {
        // shrink the filesize to the exact given size
        let _ = self.lines.pop();

        let mut length: u64 = 0;
        self.lines.iter().for_each(|s| length += s.len() as u64);

        let complement = size - length;
        for _ in 1..=complement {
            // TODO better way to fill the rest of the file until size is reached?
            self.lines.push("-");
        }

        self.to_owned()
    }

    fn collect_string(self) -> String {
        self.lines.into_par_iter().collect::<String>()
    }

    fn populate_file(self, path: PathBuf) {
        let content = self.collect_string();

        // safety check
        assert!(content.len() as u64 <= MAXSIZE);

        // WARN overrides existing files
        fs::write(path, content).unwrap();
    }
}

fn let_user_confirm() {
    // warn before producing large files
    loop {
        println!("This could produce a {} file!", "VERY LARGE".bold().red());
        println!(
            "{}",
            "Are you sure you want to exceed the default maximum filesize? [y/N]"
        );

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim() {
            "y" | "Y" => break,
            "" | "n" | "N" => {
                println!("Aborting");
                process::exit(0);
            }
            _ => continue,
        }
    }
}

// build cli
fn gerf() -> Command {
    Command::new("gerf")
        .bin_name("gerf")
        .before_help(format!(
            "{}\n{}",
            "GERF".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .about("Generate Random File Content")
        .before_long_help(format!(
            "{}\n{}",
            "GERF".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .long_about(format!(
            "{}",
            "Generate random file with a specified size and random (or not so random) file content",
        ))
        // TODO update version
        .version("1.0.5")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg(
            Arg::new("exceed")
                .short('e')
                .long("exceed")
                .help("Exceed the default maximum filesize")
                .long_help(format!(
                    "{}\n{}",
                    "Exceed the default maximum filesize",
                    "DANGER: Can produce a very large file".red(),
                ))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("gb")
                .long("gb")
                .aliases(["gigabyte", "gigabytes"])
                .help("Treat size input as gigabyte [Gb]")
                .long_help(format!(
                    "{}\n{}",
                    "Treat size input as gigabyte [Gb]", "Not as bytes [b]"
                ))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("kb")
                .long("kb")
                .aliases(["kilobyte", "kilobytes"])
                .help("Treat size input as kilobyte [Kb]")
                .long_help(format!(
                    "{}\n{}",
                    "Treat size input as kilobyte [Kb]", "Not as bytes [b]"
                ))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("mb")
                .long("mb")
                .aliases(["megabyte", "megabytes"])
                .help("Treat size input as megabyte [Mb]")
                .long_help(format!(
                    "{}\n{}",
                    "Treat size input as megabyte [Mb]", "Not as bytes [b]"
                ))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("nums")
                .short('n')
                .long("nums")
                .help("Fill the file with random numbers")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("override")
                .short('o')
                .long("override")
                .help("Override an existing file")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .alias("name")
                .default_value("gerf.txt")
                .help("Set a custom filepath / filename")
                .value_name("PATH")
                .num_args(1)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("size")
                .help("The size the generated file should have")
                .long_help(format!(
                    "{}\n{}",
                    "The size the generated file should have", "Default unit is [Bytes]",
                ))
                .value_name("SIZE")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("words")
                .short('w')
                .long("words")
                .help("Fill the file with random words")
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("examples")
                .long_flag("examples")
                .about("Show examples"),
        )
        .subcommand(
            Command::new("log")
                .short_flag('L')
                .long_flag("log")
                .about("Show content of the log file"),
        )
}

fn examples() {
    println!("{}\n----------", "Example 1".bold());
    println!(
        r###"
- generate a file with the default name 'gerf.txt' with a size of 100 Bytes

$ gerf 100     
        "###
    );

    println!("{}\n----------", "Example 2".bold());
    println!(
        r###"
- generate a file with a custom name 'wasd.md' and with a size of 12 MB

$ gerf 12 --mb --path wasd.md          
        "###
    );
}

fn check_create_config_dir() -> io::Result<PathBuf> {
    let mut new_dir = PathBuf::new();
    match dirs::config_dir() {
        Some(config_dir) => {
            new_dir.push(config_dir);
            new_dir.push("gerf");
            if !new_dir.as_path().exists() {
                fs::create_dir(&new_dir)?;
            }
        }
        None => {
            error!("Unable to find config directory");
        }
    }

    Ok(new_dir)
}

fn show_log_file(config_dir: &PathBuf) -> io::Result<String> {
    let log_path = Path::new(&config_dir).join("gerf.log");
    return match log_path.try_exists()? {
        true => Ok(format!(
            "{} {}\n{}",
            "Log location:".italic().dimmed(),
            &log_path.display(),
            fs::read_to_string(&log_path)?
        )),
        false => Ok(format!(
            "{} {}",
            "No log file found:"
                .truecolor(250, 0, 104)
                .bold()
                .to_string(),
            log_path.display()
        )),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn max_size_test() {
        let size: u64 = 5368709121; // MAXSIZE = 5368709120 == 5 GB
        assert!(size as u64 <= MAXSIZE);
    }

    #[test]
    fn create_content_new_test() {
        let vec: Vec<&str> = Vec::new();
        let content = Content::new();
        assert!(content.lines.is_empty());
        assert_eq!(content.lines, vec);
    }

    #[test]
    fn create_content_from_test() {
        let content = Content::from(vec!["one", "two", "three"]);
        assert_eq!(content.lines, vec!["one", "two", "three"]);
    }

    #[test]
    fn content_size_test() {
        let size: u64 = 1000;
        let content = Content::new()
            .genrand_content(size)
            .shrink_to_size(size)
            .collect_string();
        dbg!(&size);
        dbg!(&content.len());
        assert!(content.len() == size as usize);
    }

    #[test]
    fn content_shrink_size_test() {
        let size: u64 = 8;
        dbg!(&size);

        let content = Content::from(vec!["one", "two", "three"]).collect_string();
        dbg!(&content.len());
        assert!(content.len() != size as usize);

        let content = Content::from(vec!["one", "two", "three"])
            .shrink_to_size(size)
            .collect_string();
        dbg!(&content.len());
        assert!(content.len() == size as usize);
    }

    #[test]
    fn collect_string_test() {
        let result = Content::from(vec!["This", " ", "is", " ", "a", " ", "test"]).collect_string();
        let expect = "This is a test";
        assert_eq!(result, expect);
    }

    #[test]
    fn unit_convertion_b_test() {
        let size = Size {
            size: 123,
            unit: Unit::Byte,
        };
        let result = size.convert();
        let expect: u64 = 123; // 123 * 1024
        assert_eq!(result, expect);
    }

    #[test]
    fn unit_convertion_kb_test() {
        let size = Size {
            size: 123,
            unit: Unit::Kilobyte,
        };
        let result = size.convert();
        let expect: u64 = 125952; // 123 * 1024
        assert_eq!(result, expect);
    }

    #[test]
    fn unit_convertion_mb_test() {
        let size = Size {
            size: 123,
            unit: Unit::Megabyte,
        };
        let result = size.convert();
        let expect: u64 = 128974848; // 123 * 1024 * 1024
        assert_eq!(result, expect);
    }

    #[test]
    fn unit_convertion_gb_test() {
        let size = Size {
            size: 123,
            unit: Unit::Gigabyte,
        };
        let result = size.convert();
        let expect: u64 = 132070244352; // 123 * 1024 * 1024 * 1024
        assert_eq!(result, expect);
    }
}
