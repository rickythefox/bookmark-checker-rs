use bookmark_checker::{RunConfig, run_with_config};
use std::env;
use std::process;

const HELP: &str = r#"bookmark-checker — audit Chrome bookmarks for unreachable URLs.

USAGE:
    bookmark-checker [OPTIONS]
    bookmark-checker --list-profiles
    bookmark-checker --clean [--profile <name>]

OPTIONS:
    -m, --max-bookmarks <count>  Limit how many bookmarks to check before stopping.
    -l, --list-profiles          List detected Chrome profiles and exit.
    -p, --profile <name>         Select a profile instead of the default "Default".
    -c, --clean                  Remove bookmarks listed in bookmark_failures.yml.
    -h, --help                   Show this help text.

If run without flags, the tool checks every bookmark in the default profile.
Use --clean after a previous run created bookmark_failures.yml to prune those entries.
"#;

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}\n{HELP}");
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
            "--clean" | "-c" => {
                config.clean = true;
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

    Ok(config)
}
