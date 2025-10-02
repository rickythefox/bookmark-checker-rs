mod checker;
mod locator;
mod parser;
mod progress;

use checker::check_bookmarks;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bookmark {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkLocation {
    pub directory: PathBuf,
    pub file: PathBuf,
}

#[derive(Debug)]
pub enum BookmarkError {
    UnsupportedPlatform,
    MissingBookmarksDir(PathBuf),
    MissingBookmarksFile(PathBuf),
    Io(io::Error),
    InvalidFormat(serde_json::Error),
    HttpClientBuild(reqwest::Error),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RunConfig {
    pub max_bookmarks: Option<usize>,
}

impl Display for BookmarkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BookmarkError::UnsupportedPlatform => {
                write!(
                    f,
                    "Unsupported operating system for locating Chrome bookmarks"
                )
            }
            BookmarkError::MissingBookmarksDir(path) => {
                write!(
                    f,
                    "Chrome bookmarks directory not found: {}",
                    path.display()
                )
            }
            BookmarkError::MissingBookmarksFile(path) => {
                write!(f, "Chrome bookmarks file not found: {}", path.display())
            }
            BookmarkError::Io(err) => write!(f, "I/O error reading bookmarks: {err}"),
            BookmarkError::InvalidFormat(err) => {
                write!(f, "Failed to parse bookmarks file: {err}")
            }
            BookmarkError::HttpClientBuild(err) => {
                write!(f, "Failed to create HTTP client: {err}")
            }
        }
    }
}

impl StdError for BookmarkError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            BookmarkError::Io(err) => Some(err),
            BookmarkError::InvalidFormat(err) => Some(err),
            BookmarkError::HttpClientBuild(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for BookmarkError {
    fn from(value: io::Error) -> Self {
        BookmarkError::Io(value)
    }
}

impl From<serde_json::Error> for BookmarkError {
    fn from(value: serde_json::Error) -> Self {
        BookmarkError::InvalidFormat(value)
    }
}

pub fn run() -> Result<(), BookmarkError> {
    run_with_config(RunConfig::default())
}

pub fn run_with_config(config: RunConfig) -> Result<(), BookmarkError> {
    let (location, bookmarks) = gather_bookmarks()?;

    if bookmarks.is_empty() {
        println!("No bookmarks found in {}", location.file.display());
        return Ok(());
    }

    let mut bookmarks = bookmarks;
    let total_found = apply_limit(&mut bookmarks, config.max_bookmarks);
    let processing = bookmarks.len();

    if processing == 0 {
        println!(
            "Bookmark limit of 0 prevents checking any entries ({} total found).",
            total_found
        );
        return Ok(());
    }

    if processing == total_found {
        println!(
            "Checking {} bookmarks from {}",
            processing,
            location.file.display()
        );
    } else {
        println!(
            "Checking {} of {} bookmarks from {}",
            processing,
            total_found,
            location.file.display()
        );
    }

    let failures = check_bookmarks(&bookmarks)?;

    if failures.is_empty() {
        println!("All bookmarks responded successfully.");
    } else {
        println!("Unreachable or missing bookmarks:");
        for failure in failures {
            println!(
                "- {} ({}) -> {}",
                failure.bookmark.name, failure.bookmark.url, failure.reason
            );
        }
    }

    Ok(())
}

pub fn gather_bookmarks() -> Result<(BookmarkLocation, Vec<Bookmark>), BookmarkError> {
    let location = locator::locate()?;

    if !location.directory.exists() {
        return Err(BookmarkError::MissingBookmarksDir(
            location.directory.clone(),
        ));
    }

    if !location.file.exists() {
        return Err(BookmarkError::MissingBookmarksFile(location.file.clone()));
    }

    let bookmarks = load_bookmarks_from(&location.file)?;
    Ok((location, bookmarks))
}

fn load_bookmarks_from(path: &Path) -> Result<Vec<Bookmark>, BookmarkError> {
    let contents = fs::read_to_string(path)?;
    parser::parse_bookmarks(&contents).map_err(BookmarkError::from)
}

fn apply_limit(bookmarks: &mut Vec<Bookmark>, limit: Option<usize>) -> usize {
    let total = bookmarks.len();

    if let Some(max) = limit {
        if max < bookmarks.len() {
            bookmarks.truncate(max);
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_mentions_missing_dir() {
        let path = PathBuf::from("/tmp/does-not-exist");
        let message = BookmarkError::MissingBookmarksDir(path.clone()).to_string();
        assert!(message.contains(path.to_string_lossy().as_ref()));
    }

    #[test]
    fn load_bookmarks_propagates_parse_errors() {
        let result = parser::parse_bookmarks("not json");
        assert!(result.is_err());
    }

    #[test]
    fn limit_reduces_bookmarks_when_needed() {
        let mut bookmarks = vec![
            Bookmark {
                name: "One".into(),
                url: "https://one".into(),
            },
            Bookmark {
                name: "Two".into(),
                url: "https://two".into(),
            },
            Bookmark {
                name: "Three".into(),
                url: "https://three".into(),
            },
        ];

        let total = apply_limit(&mut bookmarks, Some(2));
        assert_eq!(total, 3);
        assert_eq!(bookmarks.len(), 2);
    }

    #[test]
    fn limit_is_noop_when_higher_than_total() {
        let mut bookmarks = vec![Bookmark {
            name: "Only".into(),
            url: "https://only".into(),
        }];

        let total = apply_limit(&mut bookmarks, Some(10));
        assert_eq!(total, 1);
        assert_eq!(bookmarks.len(), 1);
    }
}
