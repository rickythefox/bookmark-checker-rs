use crate::{BookmarkError, BookmarkLocation};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn locate() -> Result<BookmarkLocation, BookmarkError> {
    let directory = bookmarks_directory().ok_or(BookmarkError::UnsupportedPlatform)?;
    let file = bookmarks_file().ok_or(BookmarkError::UnsupportedPlatform)?;
    Ok(BookmarkLocation { directory, file })
}

pub(crate) fn bookmarks_directory() -> Option<PathBuf> {
    platform::bookmarks_dir()
}

pub(crate) fn bookmarks_file() -> Option<PathBuf> {
    platform::bookmarks_file()
}

pub(crate) fn list_profiles() -> Result<Vec<BookmarkLocation>, BookmarkError> {
    let root = profiles_root()?;
    collect_profiles_from(&root)
}

pub(crate) fn locate_profile(profile: Option<&str>) -> Result<BookmarkLocation, BookmarkError> {
    match profile {
        None => locate(),
        Some(name) => {
            let root = profiles_root()?;
            find_profile_by_name(&root, name)
        }
    }
}

fn profiles_root() -> Result<PathBuf, BookmarkError> {
    let default_dir = bookmarks_directory().ok_or(BookmarkError::UnsupportedPlatform)?;
    default_dir
        .parent()
        .map(|parent| parent.to_path_buf())
        .ok_or_else(|| BookmarkError::MissingBookmarksDir(default_dir))
}

fn collect_profiles_from(root: &Path) -> Result<Vec<BookmarkLocation>, BookmarkError> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let directory = entry.path();
            let file = directory.join("Bookmarks");
            if file.exists() {
                profiles.push(BookmarkLocation { directory, file });
            }
        }
    }

    profiles.sort_by(|a, b| a.directory.cmp(&b.directory));

    Ok(profiles)
}

fn find_profile_by_name(root: &Path, name: &str) -> Result<BookmarkLocation, BookmarkError> {
    let target = name.to_ascii_lowercase();
    let profiles = collect_profiles_from(root)?;

    profiles
        .into_iter()
        .find(|profile| {
            profile
                .directory
                .file_name()
                .and_then(|value| value.to_str())
                .map(|candidate| candidate.to_ascii_lowercase() == target)
                .unwrap_or(false)
        })
        .ok_or_else(|| BookmarkError::ProfileNotFound(name.to_string()))
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    pub(super) fn bookmarks_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|home| bookmarks_dir_from_home(home.as_path()))
    }

    pub(super) fn bookmarks_file() -> Option<PathBuf> {
        dirs::home_dir().map(|home| bookmarks_file_from_home(home.as_path()))
    }

    pub(super) fn bookmarks_dir_from_home(home: &Path) -> PathBuf {
        home.join("Library/Application Support/Google/Chrome/Default")
    }

    pub(super) fn bookmarks_file_from_home(home: &Path) -> PathBuf {
        bookmarks_dir_from_home(home).join("Bookmarks")
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn dir_and_file_are_appended_to_home() {
            let home = PathBuf::from("/Users/example");
            assert_eq!(
                bookmarks_dir_from_home(&home),
                PathBuf::from("/Users/example/Library/Application Support/Google/Chrome/Default")
            );
            assert_eq!(
                bookmarks_file_from_home(&home),
                PathBuf::from(
                    "/Users/example/Library/Application Support/Google/Chrome/Default/Bookmarks",
                )
            );
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;

    pub(super) fn bookmarks_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|home| bookmarks_dir_from_home(home.as_path()))
    }

    pub(super) fn bookmarks_file() -> Option<PathBuf> {
        dirs::home_dir().map(|home| bookmarks_file_from_home(home.as_path()))
    }

    pub(super) fn bookmarks_dir_from_home(home: &Path) -> PathBuf {
        home.join(".config/google-chrome/Default")
    }

    pub(super) fn bookmarks_file_from_home(home: &Path) -> PathBuf {
        bookmarks_dir_from_home(home).join("Bookmarks")
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn dir_and_file_are_appended_to_home() {
            let home = PathBuf::from("/home/example");
            assert_eq!(
                bookmarks_dir_from_home(&home),
                PathBuf::from("/home/example/.config/google-chrome/Default")
            );
            assert_eq!(
                bookmarks_file_from_home(&home),
                PathBuf::from("/home/example/.config/google-chrome/Default/Bookmarks")
            );
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::env;

    pub(super) fn bookmarks_dir() -> Option<PathBuf> {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|base| bookmarks_dir_from_local_app_data(base.as_path()))
    }

    pub(super) fn bookmarks_file() -> Option<PathBuf> {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|base| bookmarks_file_from_local_app_data(base.as_path()))
    }

    pub(super) fn bookmarks_dir_from_local_app_data(base: &Path) -> PathBuf {
        base.join("Google\\Chrome\\User Data\\Default")
    }

    pub(super) fn bookmarks_file_from_local_app_data(base: &Path) -> PathBuf {
        bookmarks_dir_from_local_app_data(base).join("Bookmarks")
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn dir_and_file_are_appended_to_local_app_data() {
            let base = PathBuf::from(r"C:\\Users\\example\\AppData\\Local");
            assert_eq!(
                bookmarks_dir_from_local_app_data(&base),
                PathBuf::from(
                    r"C:\\Users\\example\\AppData\\Local\\Google\\Chrome\\User Data\\Default",
                )
            );
            assert_eq!(
                bookmarks_file_from_local_app_data(&base),
                PathBuf::from(
                    r"C:\\Users\\example\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Bookmarks",
                )
            );
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
mod platform {
    use super::*;

    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn bookmarks_dir() -> Option<PathBuf> {
        None
    }

    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn bookmarks_file() -> Option<PathBuf> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn collect_profiles_includes_directories_with_bookmarks() {
        let root = temp_profile_root();
        let default_dir = root.join("Default");
        let profile_dir = root.join("Profile 1");
        let ignored_dir = root.join("System Profile");

        fs::create_dir_all(&default_dir).unwrap();
        fs::write(default_dir.join("Bookmarks"), "{}").unwrap();

        fs::create_dir_all(&profile_dir).unwrap();
        fs::write(profile_dir.join("Bookmarks"), "{}").unwrap();

        fs::create_dir_all(&ignored_dir).unwrap();

        let profiles = collect_profiles_from(&root).expect("profiles should be collected");

        assert_eq!(profiles.len(), 2);
        assert!(
            profiles
                .iter()
                .any(|profile| profile.directory == default_dir)
        );
        assert!(
            profiles
                .iter()
                .any(|profile| profile.directory == profile_dir)
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn find_profile_by_name_is_case_insensitive() {
        let root = temp_profile_root();
        let profile_dir = root.join("Profile 2");

        fs::create_dir_all(&profile_dir).unwrap();
        fs::write(profile_dir.join("Bookmarks"), "{}").unwrap();

        let location = find_profile_by_name(&root, "profile 2").expect("profile should be found");
        assert_eq!(location.directory, profile_dir);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn find_profile_by_name_errors_for_unknown_profile() {
        let root = temp_profile_root();
        let err = find_profile_by_name(&root, "Missing").expect_err("should error");
        match err {
            BookmarkError::ProfileNotFound(name) => assert_eq!(name, "Missing"),
            other => panic!("unexpected error: {other:?}"),
        }

        fs::remove_dir_all(&root).unwrap();
    }

    fn temp_profile_root() -> PathBuf {
        let mut root = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        root.push(format!("bookmark-checker-profiles-{unique}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
