use bookmark_checker::{RunConfig, VERSION, run_with_config};
use std::env;
use std::process;

const HELP: &str = r#"bookmark-checker — audit Chrome bookmarks for unreachable URLs.

USAGE:
    bookmark-checker --scan [OPTIONS]    (alias: -s)
    bookmark-checker --list-profiles
    bookmark-checker --clean [--profile <name>]

OPTIONS:
    -s, --scan                   Check bookmarks and record unreachable URLs.
    -m, --max-bookmarks <count>  Limit how many bookmarks to check before stopping.
    -l, --list-profiles          List detected Chrome profiles and exit.
    -p, --profile <name>         Select a profile instead of the default "Default".
    -c, --clean                  Remove bookmarks listed in bookmark_failures.yml.
    -V, -v, --version            Print the app version and exit.
    -h, --help                   Show this help text.

GUIDE:
    - Run `bookmark-checker --scan` (or `-s`) to audit bookmarks.
    - Use `--max-bookmarks` with `--scan` to limit the number checked.
    - Run `--clean` after a scan writes bookmark_failures.yml to prune entries.
    - Use `--list-profiles` to discover Chrome profiles before scanning.
    - Run without flags or use `--help` anytime to view this message again.
"#;

fn main() {
    if env::args().len() == 1 {
        println!("{HELP}");
        return;
    }

    let config = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}\n{HELP}");
            process::exit(2);
        }
    };

    if config.show_version {
        println!("{}", VERSION);
        return;
    }

    if let Err(err) = run_with_config(config) {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn parse_args() -> Result<RunConfig, String> {
    let mut args = env::args().skip(1);
    let mut config = RunConfig::default();
    config.scan = false;

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
            "--clean" | "-c" => {
                config.clean = true;
            }
            "--scan" | "-s" => {
                config.scan = true;
            }
            "--version" | "-V" | "-v" => {
                config.show_version = true;
            }
            "--help" | "-h" => {
                println!("{HELP}");
                process::exit(0);
            }
            unknown => {
                return Err(format!("Unknown argument '{unknown}'"));
            }
        }
    }

    if config.clean && config.list_profiles {
        return Err("--clean cannot be combined with --list-profiles".into());
    }

    if config.scan && config.clean {
        return Err("--scan cannot be combined with --clean".into());
    }

    if config.scan && config.list_profiles {
        return Err("--scan cannot be combined with --list-profiles".into());
    }

    if config.show_version
        && (config.clean
            || config.list_profiles
            || config.max_bookmarks.is_some()
            || config.profile.is_some()
            || config.scan)
    {
        return Err("--version cannot be combined with other options".into());
    }

    if config.max_bookmarks.is_some() && !config.scan {
        return Err("--max-bookmarks requires --scan".into());
    }

    if config.profile.is_some() && !config.scan && !config.clean {
        return Err("--profile requires --scan or --clean".into());
    }

    if !config.scan && !config.clean && !config.list_profiles && !config.show_version {
        // Without a primary action this should have been caught earlier. Treat as misuse.
        return Err(
            "No action provided. Use --scan, --clean, --list-profiles, or --version.".into(),
        );
    }

    Ok(config)
}
