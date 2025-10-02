use crate::{BookmarkError, BookmarkLocation};
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
