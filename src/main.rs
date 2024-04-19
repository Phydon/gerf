use clap::{Arg, ArgAction, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::{error, info, warn};
use owo_colors::colored::*;

use std::{
    fs, io,
    path::{Path, PathBuf},
    process,
};

// TODO what could be a good default maximum filesize?
const MAXSIZE: u32 = 100000;
const LOREM: &str = "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquid ex ea commodi consequat Quis aute iure reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur Excepteur sint obcaecat cupiditat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum";

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
            let mut size = 0;
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
                if size > MAXSIZE {
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
            "Are you sure to exceed the default maximum filesize? [y/N]"
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

fn create_file(path: &str) {
    todo!();
}

fn populate_file(path: &str) {
    todo!();
}

// TODO generate different "random" content
// TODO generate content with only numbers
// TODO generate content with only words
// TODO generate alphanumeric content
fn generate_random_filecontent(size: u64) -> String {
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
            Arg::new("size")
                .help("The size the generated file should have")
                .long_help(format!(
                    "{}\n{}",
                    "The size the generated file should have", "Default unit is [Bytes]",
                ))
                .action(ArgAction::Set)
                .value_name("SIZE"),
        )
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
}
