use bookmark_checker::{RunConfig, run_with_config};
use std::env;
use std::process;

const USAGE: &str =
    "Usage: bookmark-checker [--max-bookmarks <count>] [--list-profiles] [--profile <name>]";

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            process::exit(2);
        }
    };

    if let Err(err) = run_with_config(config) {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn parse_args() -> Result<RunConfig, String> {
    let mut args = env::args().skip(1);
    let mut config = RunConfig::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--max-bookmarks" | "-m" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--max-bookmarks requires a numerical value".to_string())?;
                let parsed = value.parse::<usize>().map_err(|_| {
                    format!(
                        "Invalid max bookmark count '{value}'. Expected a non-negative integer."
                    )
                })?;
                config.max_bookmarks = Some(parsed);
            }
            "--list-profiles" | "-l" => {
                config.list_profiles = true;
            }
            "--profile" | "-p" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--profile requires a profile name".to_string())?;
                config.profile = Some(value);
            }
            "--help" | "-h" => {
                println!("{USAGE}");
                process::exit(0);
            }
            unknown => {
                return Err(format!("Unknown argument '{unknown}'"));
            }
        }
    }

    Ok(config)
}
