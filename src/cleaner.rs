use crate::model::{BookmarkError, BookmarkLocation};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct CleanupResult {
    pub removed: usize,
    pub backup_path: Option<PathBuf>,
}

pub(crate) fn clean_failures(
    location: &BookmarkLocation,
    report_path: &Path,
) -> Result<CleanupResult, BookmarkError> {
    if !report_path.exists() {
        return Ok(CleanupResult::default());
    }

    let report_contents = fs::read_to_string(report_path)?;
    let report: FailureReport =
        serde_yaml::from_str(&report_contents).map_err(BookmarkError::ReportParse)?;

    let targets = report.into_targets();
    if targets.is_empty() {
        return Ok(CleanupResult::default());
    }

    let backup_path = create_backup(&location.file)?;
    let mut data: Value = serde_json::from_str(&fs::read_to_string(&location.file)?)?;
    let removed = remove_targets(&mut data, &targets);

    if removed > 0 {
        let updated =
            serde_json::to_string_pretty(&data).map_err(BookmarkError::BookmarkSerialization)?;
        fs::write(&location.file, updated)?;
    }

    Ok(CleanupResult {
        removed,
        backup_path: Some(backup_path),
    })
}

fn create_backup(bookmarks_file: &Path) -> Result<PathBuf, BookmarkError> {
    let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S");
    let file_name = bookmarks_file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Bookmarks".to_string());
    let backup_name = format!("{file_name}-{timestamp}.bak");
    let backup_path = bookmarks_file.with_file_name(backup_name);
    fs::copy(bookmarks_file, &backup_path)?;
    Ok(backup_path)
}

fn remove_targets(root: &mut Value, targets: &HashSet<String>) -> usize {
    let (removed, _) = remove_node(root, targets);
    removed
}

fn remove_node(node: &mut Value, targets: &HashSet<String>) -> (usize, bool) {
    match node {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("url")
                && let Some(url) = map.get("url").and_then(Value::as_str)
                && targets.contains(url)
            {
                return (1, true);
            }

            let mut removed = 0;

            if let Some(Value::Array(children)) = map.get_mut("children") {
                let mut index = 0;
                while index < children.len() {
                    let (child_removed, should_remove_child) =
                        remove_node(&mut children[index], targets);
                    removed += child_removed;
                    if should_remove_child {
                        children.remove(index);
                    } else {
                        index += 1;
                    }
                }
            }

            let mut keys_to_remove = Vec::new();
            for (key, value) in map.iter_mut() {
                if key == "children" {
                    continue;
                }

                let (child_removed, should_remove_child) = remove_node(value, targets);
                removed += child_removed;
                if should_remove_child {
                    keys_to_remove.push(key.clone());
                }
            }

            for key in keys_to_remove {
                map.remove(&key);
            }

            (removed, false)
        }
        Value::Array(array) => {
            let mut removed = 0;
            let mut index = 0;
            while index < array.len() {
                let (child_removed, should_remove_child) = remove_node(&mut array[index], targets);
                removed += child_removed;
                if should_remove_child {
                    array.remove(index);
                } else {
                    index += 1;
                }
            }

            (removed, false)
        }
        _ => (0, false),
    }
}

#[derive(Debug, Default, Deserialize)]
struct FailureReport {
    #[serde(default)]
    not_found: Vec<FailureEntry>,
    #[serde(default)]
    unauthorized: Vec<FailureEntry>,
    #[serde(default)]
    connection_errors: Vec<FailureEntry>,
}

impl FailureReport {
    fn into_targets(self) -> HashSet<String> {
        self.not_found
            .into_iter()
            .chain(self.unauthorized)
            .chain(self.connection_errors)
            .filter_map(|entry| entry.url)
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct FailureEntry {
    #[serde(default)]
    url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BookmarkLocation;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn removes_bookmarks_listed_in_report() {
        let temp_dir = temp_dir();
        let bookmarks_path = temp_dir.join("Bookmarks");
        fs::write(&bookmarks_path, sample_bookmarks_json()).unwrap();

        let report_path = temp_dir.join("bookmark_failures.yml");
        fs::write(&report_path, sample_report_yaml()).unwrap();

        let location = BookmarkLocation {
            directory: temp_dir.clone(),
            file: bookmarks_path.clone(),
        };

        let result = clean_failures(&location, &report_path).expect("clean");
        assert_eq!(result.removed, 1);
        assert!(result.backup_path.unwrap().exists());

        let updated = fs::read_to_string(&bookmarks_path).unwrap();
        assert!(updated.contains("https://keep.me"));
        assert!(!updated.contains("https://remove.me"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn no_report_returns_zero_without_backup() {
        let temp_dir = temp_dir();
        let bookmarks_path = temp_dir.join("Bookmarks");
        fs::write(&bookmarks_path, sample_bookmarks_json()).unwrap();

        let location = BookmarkLocation {
            directory: temp_dir.clone(),
            file: bookmarks_path.clone(),
        };

        let result = clean_failures(&location, &temp_dir.join("missing.yml")).expect("clean");
        assert_eq!(result.removed, 0);
        assert!(result.backup_path.is_none());

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn preserves_bookmarks_when_no_match_found() {
        let temp_dir = temp_dir();
        let bookmarks_path = temp_dir.join("Bookmarks");
        fs::write(&bookmarks_path, sample_bookmarks_json()).unwrap();

        let report_path = temp_dir.join("bookmark_failures.yml");
        fs::write(&report_path, sample_report_without_match()).unwrap();

        let original = fs::read_to_string(&bookmarks_path).unwrap();

        let location = BookmarkLocation {
            directory: temp_dir.clone(),
            file: bookmarks_path.clone(),
        };

        let result = clean_failures(&location, &report_path).expect("clean");
        assert_eq!(result.removed, 0);
        assert!(result.backup_path.is_some());

        let updated = fs::read_to_string(&bookmarks_path).unwrap();
        assert_eq!(updated, original);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    fn temp_dir() -> PathBuf {
        let mut dir = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("bookmark-cleaner-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_bookmarks_json() -> &'static str {
        r#"{
            "roots": {
                "bookmark_bar": {
                    "children": [
                        {
                            "type": "url",
                            "name": "Keep",
                            "url": "https://keep.me"
                        },
                        {
                            "type": "url",
                            "name": "Remove",
                            "url": "https://remove.me"
                        }
                    ]
                }
            }
        }"#
    }

    fn sample_report_yaml() -> &'static str {
        "not_found:\n  - name: Remove\n    url: https://remove.me\n    reason: HTTP 404 Not Found\n"
    }

    fn sample_report_without_match() -> &'static str {
        "not_found:\n  - name: Missing One\n    url: https://missing.me\n"
    }
}
