use crate::{Bookmark, BookmarkError, progress::ProgressReporter};
use rayon::prelude::*;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub(crate) struct LinkFailure {
    pub(crate) bookmark: Bookmark,
    pub(crate) reason: String,
}

pub(crate) fn check_bookmarks(bookmarks: &[Bookmark]) -> Result<Vec<LinkFailure>, BookmarkError> {
    if bookmarks.is_empty() {
        return Ok(Vec::new());
    }

    let client = build_client()?;
    let total = bookmarks.len();
    let worker_count = rayon::current_num_threads();
    let reporter = ProgressReporter::new(total, worker_count, "Checking bookmarks");
    let handle = reporter.handle();

    let failures: Vec<LinkFailure> = bookmarks
        .par_iter()
        .map_init(
            || handle.clone(),
            |progress, bookmark| {
                if let Some(idx) = rayon::current_thread_index() {
                    progress.worker_start(idx, format!("{} -> {}", bookmark.name, bookmark.url));
                }

                let result = check_single(bookmark, &client);

                progress.inc();

                if let Some(idx) = rayon::current_thread_index() {
                    progress.worker_finish(idx);
                }

                result
            },
        )
        .filter_map(|failure| failure)
        .collect();

    reporter.finish();

    Ok(failures)
}

fn build_client() -> Result<Client, BookmarkError> {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(BookmarkError::HttpClientBuild)
}

fn check_single(bookmark: &Bookmark, client: &Client) -> Option<LinkFailure> {
    match client.get(&bookmark.url).send() {
        Ok(response) => {
            if response.status() == StatusCode::NOT_FOUND {
                Some(LinkFailure::from_status(bookmark, response.status()))
            } else {
                None
            }
        }
        Err(err) => Some(LinkFailure::from_error(bookmark, &err)),
    }
}

impl LinkFailure {
    fn from_status(bookmark: &Bookmark, status: StatusCode) -> Self {
        let canonical = status.canonical_reason().unwrap_or("Unknown");
        Self {
            bookmark: bookmark.clone(),
            reason: format!("HTTP {} {}", status.as_u16(), canonical),
        }
    }

    fn from_error(bookmark: &Bookmark, err: &reqwest::Error) -> Self {
        Self {
            bookmark: bookmark.clone(),
            reason: format!("Request failed: {err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_no_failures() {
        let result = check_bookmarks(&[]).expect("should succeed");
        assert!(result.is_empty());
    }

    #[test]
    fn failure_carries_reason_for_status() {
        let bookmark = Bookmark {
            name: "Example".into(),
            url: "https://example".into(),
        };

        let failure = LinkFailure::from_status(&bookmark, StatusCode::NOT_FOUND);
        assert_eq!(failure.reason, "HTTP 404 Not Found");
        assert_eq!(failure.bookmark.url, bookmark.url);
    }
}
