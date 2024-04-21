// TODO remove later
#![allow(dead_code)]

use clap::{Arg, ArgAction, Command};
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

// TODO what could be a good default maximum filesize?
const MAXSIZE: u32 = 64 * (1 << 10); // 64 KB
const LOREM: [&str; 12] = [
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
    let override_flag = matches.get_flag("override");
    // let unit_flag = matches.get_flag("unit");

    if let Some(_) = matches.subcommand_matches("log") {
        if let Ok(logs) = show_log_file(&config_dir) {
            println!("{}", "Available logs:".bold().yellow());
            println!("{}", logs);
        } else {
            error!("Unable to read logs");
            process::exit(1);
        }
    } else {
        if let Some(s) = matches.get_one::<String>("size") {
            let mut size: u64 = 0;
            match s.parse() {
                Ok(s) => size = s,
                Err(err) => {
                    warn!("Expected a number as filesize: {err}");
                    process::exit(1);
                }
            }

            // TODO accept different units fpr the filesize
            // TODO default: Bytes; other: KB, MB, GB

            if !exceed_flag {
                // make sure the user doesn't accidentally produces hugh files
                if size > MAXSIZE as u64 {
                    warn!(
                        "Size '{}' exceeds the default maximum filesize of '{}'",
                        size, MAXSIZE
                    );
                    info!(
                            "Use the [ -e ] or [ --exceed ] flag to exceed the default maximum filesize"
                    );
                    process::exit(0);
                }
            } else {
                // let user confirm before producing big files
                let_user_confirm();
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

            // create file of given size and file with content
            let mut content = generate_random_filecontent(size);
            let content = shrink_content_to_exact_size(&mut content, size);
            let content = make_string(content);
            populate_file(path, content);
        } else {
            let _ = gerf().print_help();
            process::exit(0);
        }
    }
}

fn let_user_confirm() {
    loop {
        println!("This could produce {} files!", "VERY LARGE".bold().red());
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

// TODO generate different "random" content
// TODO generate content with only numbers
// TODO generate content with only words
// TODO generate alphanumeric content
fn generate_random_filecontent(size: u64) -> Vec<&'static str> {
    let mut content: Vec<&str> = Vec::new();
    let mut rng = thread_rng();
    let mut length: u64 = 0;

    for _ in 1..size {
        length += content.last().unwrap_or(&"").len() as u64;

        if length >= size {
            break;
        }

        let i: u8 = rng.gen_range(0..=(LOREM.len() - 1) as u8);
        content.push(LOREM[i as usize]);
    }

    content
}

fn shrink_content_to_exact_size(content: &mut Vec<&'static str>, size: u64) -> Vec<&'static str> {
    let _ = content.pop();

    let mut length: u64 = 0;
    content.into_iter().for_each(|s| length += s.len() as u64);

    let complement = size - length;
    for _ in 1..=complement {
        // TODO better way to fill the rest of the file until size is reached?
        content.push("-");
    }

    // TODO better way than cloning?
    content.clone()
}

fn make_string(content: Vec<&'static str>) -> String {
    content.into_par_iter().collect::<String>()
}

fn populate_file(path: PathBuf, content: String) {
    // WARN overrides existing files
    fs::write(path, content).unwrap();
}

fn convert_size() {
    todo!();
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
        .version("1.0.0")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg(
            Arg::new("exceed")
                .short('e')
                .long("exceed")
                .help("Exceed the default maximum filesize")
                .long_help(format!(
                    "{}\n{}",
                    "Exceed the default maximum filesize",
                    "DANGER: Can produce very large files".red(),
                ))
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
        .subcommand(
            Command::new("log")
                .short_flag('L')
                .long_flag("log")
                .about("Show content of the log file"),
        )
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
    fn maximum_size_test() {
        assert!(100000 < MAXSIZE);
    }

    #[test]
    fn content_size_test() {
        let size: u64 = 1000;
        let mut content = generate_random_filecontent(size);
        let content = shrink_content_to_exact_size(&mut content, size);
        let content = make_string(content);
        dbg!(&size);
        dbg!(&content.len());
        assert!(content.len() == size as usize);
    }

    #[test]
    fn make_string_test() {
        let inp = vec!["This", " ", "is", " ", "a", " ", "test"];
        let result = make_string(inp);
        let expect = "This is a test";
        assert_eq!(result, expect);
    }
}
