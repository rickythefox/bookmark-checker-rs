use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;

pub struct ProgressReporter {
    multi: MultiProgress,
    inner: Arc<ProgressInner>,
}

#[derive(Clone)]
pub struct ProgressHandle {
    inner: Arc<ProgressInner>,
}

struct ProgressInner {
    overall: ProgressBar,
    workers: Vec<ProgressBar>,
}

impl ProgressReporter {
    pub fn new(total: usize, worker_count: usize, label: &str) -> Self {
        let multi = MultiProgress::new();
        let overall = create_overall_bar(&multi, total, label);
        let workers = (0..worker_count)
            .map(|idx| create_worker_bar(&multi, idx))
            .collect();

        Self {
            multi,
            inner: Arc::new(ProgressInner { overall, workers }),
        }
    }

    pub fn handle(&self) -> ProgressHandle {
        ProgressHandle {
            inner: Arc::clone(&self.inner),
        }
    }

    pub fn finish(self) {
        self.inner.overall.finish_and_clear();
        for worker in &self.inner.workers {
            worker.finish_and_clear();
        }
        let _ = self.multi.clear();
    }
}

impl ProgressHandle {
    pub fn inc(&self) {
        self.inner.overall.inc(1);
    }

    pub fn worker_start(&self, idx: usize, message: impl AsRef<str>) {
        if let Some(bar) = self.inner.workers.get(idx) {
            bar.set_message(message.as_ref().to_string());
        }
    }

    pub fn worker_finish(&self, idx: usize) {
        if let Some(bar) = self.inner.workers.get(idx) {
            bar.set_message("idle".to_string());
        }
    }
}

fn create_overall_bar(multi: &MultiProgress, total: usize, label: &str) -> ProgressBar {
    let bar = multi.add(ProgressBar::new(total as u64));
    bar.set_style(
        ProgressStyle::with_template("{prefix} {bar:40.cyan/blue} {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("=>-"),
    );
    bar.set_prefix(label.to_string());
    bar.enable_steady_tick(Duration::from_millis(100));
    bar
}

fn create_worker_bar(multi: &MultiProgress, idx: usize) -> ProgressBar {
    let bar = multi.add(ProgressBar::new_spinner());
    bar.set_style(
        ProgressStyle::with_template("Thread {prefix}: {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    bar.set_prefix(idx.to_string());
    bar.set_message("idle".to_string());
    bar.enable_steady_tick(Duration::from_millis(100));
    bar
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_updates_overall_count() {
        let reporter = ProgressReporter::new(3, 1, "Testing");
        let handle = reporter.handle();
        handle.inc();
        assert_eq!(handle.inner.overall.position(), 1);
        reporter.finish();
    }
}
