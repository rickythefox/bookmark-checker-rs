use crate::checker::{FailureKind, LinkFailure};
use crate::model::BookmarkError;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const FAILURE_REPORT_FILE: &str = "bookmark_failures.yml";

#[derive(Debug)]
pub struct FailureReporter {
    output_path: PathBuf,
}

impl FailureReporter {
    pub fn new<P: Into<PathBuf>>(output_path: P) -> Self {
        Self {
            output_path: output_path.into(),
        }
    }

    pub fn default() -> Self {
        Self::new(FAILURE_REPORT_FILE)
    }

    pub fn write_report(&self, failures: &[LinkFailure]) -> Result<(), BookmarkError> {
        let report = FailureReport::from_failures(failures);
        let yaml = serde_yaml::to_string(&report)?;
        fs::write(&self.output_path, yaml)?;
        Ok(())
    }

    pub fn output_path(&self) -> &Path {
        &self.output_path
    }
}

#[derive(Debug, Serialize)]
struct FailureReport {
    not_found: Vec<ReportEntry>,
    unauthorized: Vec<ReportEntry>,
    connection_errors: Vec<ReportEntry>,
}

impl FailureReport {
    fn from_failures(failures: &[LinkFailure]) -> Self {
        let mut not_found = Vec::new();
        let mut unauthorized = Vec::new();
        let mut connection_errors = Vec::new();

        for failure in failures {
            let entry = ReportEntry::from(failure);
            match failure.kind {
                FailureKind::NotFound => not_found.push(entry),
                FailureKind::Unauthorized => unauthorized.push(entry),
                FailureKind::Connection => connection_errors.push(entry),
            }
        }

        Self {
            not_found,
            unauthorized,
            connection_errors,
        }
    }
}

#[derive(Debug, Serialize)]
struct ReportEntry {
    name: String,
    url: String,
    reason: String,
}

impl From<&LinkFailure> for ReportEntry {
    fn from(value: &LinkFailure) -> Self {
        Self {
            name: value.bookmark.name.clone(),
            url: value.bookmark.url.clone(),
            reason: value.reason.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Bookmark;

    fn bookmark(name: &str, url: &str) -> Bookmark {
        Bookmark {
            name: name.into(),
            url: url.into(),
        }
    }

    #[test]
    fn report_groups_failures_by_kind() {
        let failures = vec![
            LinkFailure {
                bookmark: bookmark("Missing", "https://example.com/missing"),
                reason: "HTTP 404 Not Found".into(),
                kind: FailureKind::NotFound,
            },
            LinkFailure {
                bookmark: bookmark("Private", "https://example.com/private"),
                reason: "HTTP 403 Forbidden".into(),
                kind: FailureKind::Unauthorized,
            },
            LinkFailure {
                bookmark: bookmark("Timeout", "https://example.com/timeout"),
                reason: "Request failed: timeout".into(),
                kind: FailureKind::Connection,
            },
        ];

        let report = FailureReport::from_failures(&failures);
        assert_eq!(report.not_found.len(), 1);
        assert_eq!(report.unauthorized.len(), 1);
        assert_eq!(report.connection_errors.len(), 1);
    }

    #[test]
    fn reporter_writes_yaml_to_disk() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "bookmark-checker-report-{}.yml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let reporter = FailureReporter::new(&path);
        let failures = vec![LinkFailure {
            bookmark: bookmark("Missing", "https://example.com/missing"),
            reason: "HTTP 404 Not Found".into(),
            kind: FailureKind::NotFound,
        }];

        reporter.write_report(&failures).expect("write");

        let contents = fs::read_to_string(&path).expect("read");
        assert!(contents.contains("not_found"));
        assert!(contents.contains("https://example.com/missing"));

        let _ = fs::remove_file(path);
    }
}
