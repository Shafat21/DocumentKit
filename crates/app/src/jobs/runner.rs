//! Tool runner: the only way the UI executes a tool.
//!
//! Guarantees:
//! - the tool runs on its own named thread; the UI never blocks,
//! - a panicking tool becomes `Error::Internal`; the app keeps running,
//! - progress is throttled (≤ 20 updates/s) through a bounded channel, so a
//!   10 000-page job cannot flood the renderer or grow memory,
//! - cancel sets a flag the tool checks per page.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dioxus::prelude::*;
use dk_core::{Error, Job, JobOutput, Progress, Result, Tool};
use futures_channel::mpsc;
use futures_util::StreamExt;

const PROGRESS_BUFFER: usize = 16;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(50);

/// What the UI renders for a running job.
#[derive(Debug, Clone)]
pub enum JobState {
    Running {
        pct: u8,
        msg: String,
    },
    /// `Arc` so re-renders don't clone the output.
    Done(Arc<Result<JobOutput>>),
}

/// Returned by [`start`]; keep it in the tool page's state.
#[derive(Clone)]
pub struct JobHandle {
    pub state: Signal<JobState>,
    cancel: Arc<AtomicBool>,
}

impl JobHandle {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// Start `tool` on `job`. Call from a component or event handler.
pub fn start(tool: Arc<dyn Tool>, job: Job) -> JobHandle {
    let cancel = job.cancel_handle();
    let mut state = Signal::new(JobState::Running {
        pct: 0,
        msg: String::new(),
    });
    let (progress_tx, mut progress_rx) = mpsc::channel(PROGRESS_BUFFER);
    let (done_tx, done_rx) = futures_channel::oneshot::channel();

    let id = tool.spec().id;
    let spawned = std::thread::Builder::new()
        .name(format!("job-{id}"))
        .spawn(move || {
            let sink = ChannelProgress::new(progress_tx, PROGRESS_INTERVAL);
            let _ = done_tx.send(execute(tool.as_ref(), job, &sink));
            // `sink` drops here, closing the progress stream.
        });

    match spawned {
        Ok(_) => {
            spawn(async move {
                while let Some((pct, msg)) = progress_rx.next().await {
                    state.set(JobState::Running { pct, msg });
                }
                let result = done_rx.await.unwrap_or_else(|_| {
                    Err(Error::Internal("job thread ended without a result".into()))
                });
                state.set(JobState::Done(Arc::new(result)));
            });
        }
        Err(e) => state.set(JobState::Done(Arc::new(Err(Error::Io(e))))),
    }

    JobHandle { state, cancel }
}

/// Run a tool, converting a panic into `Error::Internal`. Pure; unit-testable.
pub fn execute(tool: &dyn Tool, job: Job, progress: &dyn Progress) -> Result<JobOutput> {
    let id = tool.spec().id;
    tracing::info!(tool = id, inputs = job.inputs.len(), "job started");
    let result = catch_unwind(AssertUnwindSafe(|| tool.run(job, progress)))
        .unwrap_or_else(|panic| Err(Error::Internal(panic_message(panic.as_ref()))));
    match &result {
        Ok(out) => tracing::info!(tool = id, files = out.files.len(), "job finished"),
        Err(e) => tracing::warn!(tool = id, "job failed: {e}"),
    }
    result
}

fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
    panic
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "tool panicked".into())
}

/// Throttled, bounded progress sink. Drops intermediate updates rather than blocking the tool.
type ProgressTx = mpsc::Sender<(u8, String)>;

struct ChannelProgress {
    inner: Mutex<(ProgressTx, Option<Instant>)>,
    interval: Duration,
}

impl ChannelProgress {
    fn new(tx: mpsc::Sender<(u8, String)>, interval: Duration) -> Self {
        Self {
            inner: Mutex::new((tx, None)),
            interval,
        }
    }
}

impl Progress for ChannelProgress {
    fn report(&self, pct: u8, msg: &str) {
        let Ok(mut guard) = self.inner.lock() else {
            return;
        };
        let (tx, last) = &mut *guard;
        let due = last.is_none_or(|t| t.elapsed() >= self.interval) || pct >= 100;
        if due && tx.try_send((pct.min(100), msg.to_owned())).is_ok() {
            *last = Some(Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dk_core::{catalog, NoProgress, ToolSpec};
    use std::path::PathBuf;

    struct Panics;
    impl Tool for Panics {
        fn spec(&self) -> &'static ToolSpec {
            &catalog::MERGE_PDF
        }
        fn run(&self, _: Job, _: &dyn Progress) -> Result<JobOutput> {
            panic!("boom")
        }
    }

    fn job() -> Job {
        Job::new(vec![], PathBuf::from("out"))
    }

    #[test]
    fn panic_becomes_internal_error() {
        let r = execute(&Panics, job(), &NoProgress);
        assert!(matches!(r, Err(Error::Internal(m)) if m == "boom"));
    }

    #[test]
    fn stub_tool_reports_not_implemented() {
        let r = execute(&dk_organize::SplitPdf, job(), &NoProgress);
        assert!(matches!(r, Err(Error::NotImplemented("split-pdf"))));
    }

    #[test]
    #[allow(deprecated)] // try_next is renamed in newer futures-channel; either works here.
    fn progress_is_throttled_and_bounded() {
        let (tx, mut rx) = mpsc::channel(PROGRESS_BUFFER);
        // Huge interval: the burst always fits in one window, however slow the machine.
        let sink = ChannelProgress::new(tx, Duration::from_secs(3600));
        for i in 0..10_000u32 {
            sink.report((i % 100) as u8, "page");
        }
        sink.report(100, "done");
        drop(sink);
        let mut received = 0;
        while let Ok(Some(_)) = rx.try_next() {
            received += 1;
        }
        // First update + final 100% only.
        assert_eq!(received, 2);
    }
}
