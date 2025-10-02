use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;

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
    ProfileNotFound(String),
    Io(io::Error),
    InvalidFormat(serde_json::Error),
    BookmarkSerialization(serde_json::Error),
    HttpClientBuild(reqwest::Error),
    ReportWrite(serde_yaml::Error),
    ReportParse(serde_yaml::Error),
}

#[derive(Debug, Clone, Default)]
pub struct RunConfig {
    pub max_bookmarks: Option<usize>,
    pub list_profiles: bool,
    pub profile: Option<String>,
    pub clean: bool,
    pub show_version: bool,
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
            BookmarkError::ProfileNotFound(name) => {
                write!(f, "Chrome profile '{name}' not found")
            }
            BookmarkError::Io(err) => write!(f, "I/O error reading bookmarks: {err}"),
            BookmarkError::InvalidFormat(err) => {
                write!(f, "Failed to parse bookmarks file: {err}")
            }
            BookmarkError::BookmarkSerialization(err) => {
                write!(f, "Failed to serialize bookmarks file: {err}")
            }
            BookmarkError::HttpClientBuild(err) => {
                write!(f, "Failed to create HTTP client: {err}")
            }
            BookmarkError::ReportWrite(err) => {
                write!(f, "Failed to write YAML report: {err}")
            }
            BookmarkError::ReportParse(err) => {
                write!(f, "Failed to parse YAML report: {err}")
            }
        }
    }
}

impl StdError for BookmarkError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            BookmarkError::Io(err) => Some(err),
            BookmarkError::InvalidFormat(err) => Some(err),
            BookmarkError::BookmarkSerialization(err) => Some(err),
            BookmarkError::HttpClientBuild(err) => Some(err),
            BookmarkError::ReportWrite(err) => Some(err),
            BookmarkError::ReportParse(err) => Some(err),
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

impl From<serde_yaml::Error> for BookmarkError {
    fn from(value: serde_yaml::Error) -> Self {
        BookmarkError::ReportWrite(value)
    }
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
    fn profile_not_found_error_message_includes_name() {
        let message = BookmarkError::ProfileNotFound("Profile 42".into()).to_string();
        assert!(message.contains("Profile 42"));
    }
}
