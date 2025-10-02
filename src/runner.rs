use crate::checker::check_bookmarks;
use crate::model::{Bookmark, BookmarkError, BookmarkLocation, RunConfig};
use crate::report::FailureReporter;
use crate::{locator, parser};
use std::fs;
use std::path::Path;

pub fn run() -> Result<(), BookmarkError> {
    run_with_config(RunConfig::default())
}

pub fn run_with_config(config: RunConfig) -> Result<(), BookmarkError> {
    if config.list_profiles {
        print_available_profiles()?;
        return Ok(());
    }

    let (location, mut bookmarks) = gather_bookmarks_for_profile(config.profile.as_deref())?;

    if bookmarks.is_empty() {
        println!("No bookmarks found in {}", location.file.display());
        return Ok(());
    }

    let total_found = apply_limit(&mut bookmarks, config.max_bookmarks);
    let processing = bookmarks.len();

    if processing == 0 {
        println!(
            "Bookmark limit of 0 prevents checking any entries ({} total found).",
            total_found
        );
        return Ok(());
    }

    announce_workload(total_found, processing, &location);

    let failures = check_bookmarks(&bookmarks)?;

    if failures.is_empty() {
        println!("All bookmarks responded successfully.");
    } else {
        let reporter = FailureReporter::default();
        reporter.write_report(&failures)?;
        println!(
            "Logged {} unreachable bookmarks to {}",
            failures.len(),
            reporter.output_path().display()
        );
    }

    Ok(())
}

pub fn gather_bookmarks() -> Result<(BookmarkLocation, Vec<Bookmark>), BookmarkError> {
    gather_bookmarks_for_profile(None)
}

pub fn gather_bookmarks_for_profile(
    profile: Option<&str>,
) -> Result<(BookmarkLocation, Vec<Bookmark>), BookmarkError> {
    let location = locator::locate_profile(profile)?;

    ensure_location_exists(&location)?;

    let bookmarks = load_bookmarks_from(&location.file)?;
    Ok((location, bookmarks))
}

fn ensure_location_exists(location: &BookmarkLocation) -> Result<(), BookmarkError> {
    if !location.directory.exists() {
        return Err(BookmarkError::MissingBookmarksDir(
            location.directory.clone(),
        ));
    }

    if !location.file.exists() {
        return Err(BookmarkError::MissingBookmarksFile(location.file.clone()));
    }

    Ok(())
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

fn announce_workload(total_found: usize, processing: usize, location: &BookmarkLocation) {
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
}

fn print_available_profiles() -> Result<(), BookmarkError> {
    let profiles = locator::list_profiles()?;

    if profiles.is_empty() {
        println!("No Chrome profiles with bookmarks found.");
    } else {
        println!("Available Chrome profiles:");
        for location in profiles {
            let name = location
                .directory
                .file_name()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_else(|| location.directory.display().to_string());

            println!("- {name} ({})", location.file.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
