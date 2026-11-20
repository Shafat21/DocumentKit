//! Development roadmap data, parsed from `roadmap.json` at the repo root.
//!
//! The JSON comes from the network, so it is untrusted: size-capped, parsed into
//! typed structs, and validated before the UI sees it. Unknown status strings fall
//! back to `Pending` rather than failing the whole file.

use serde::Deserialize;

use crate::{Error, Result};

/// Largest accepted roadmap.json, in bytes.
pub const MAX_BYTES: usize = 256 * 1024;
const MAX_MODULES: usize = 64;
const MAX_TASKS: usize = 1_000;
const MAX_TEXT: usize = 200;

/// Whole roadmap.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Roadmap {
    /// Project phase label, e.g. "Planning phase".
    #[serde(default)]
    pub phase: String,
    /// Date the file was last edited (free text, usually ISO date).
    #[serde(default)]
    pub updated: String,
    /// Modules in display order.
    pub modules: Vec<RoadmapModule>,
}

/// One feature module and its tasks.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RoadmapModule {
    /// Matches `ModuleId::key()`, e.g. "organize".
    pub key: String,
    /// Display name.
    pub name: String,
    /// Tasks (tools) in this module.
    pub tasks: Vec<Task>,
}

/// One tool or feature to build.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Task {
    /// Matches a catalog tool id, e.g. "merge-pdf".
    pub id: String,
    /// Display name.
    pub name: String,
    /// Person working on it; `None` = unassigned.
    #[serde(default)]
    pub assignee: Option<String>,
    /// Progress.
    #[serde(default)]
    pub status: Status,
}

/// Task progress.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(from = "String")]
pub enum Status {
    /// Not started.
    #[default]
    Pending,
    /// Being worked on.
    InProgress,
    /// Shipped.
    Done,
}

impl From<String> for Status {
    fn from(value: String) -> Self {
        match value.as_str() {
            "done" => Status::Done,
            "in_progress" | "in-progress" => Status::InProgress,
            _ => Status::Pending,
        }
    }
}

impl Status {
    /// Human-readable label.
    pub const fn label(self) -> &'static str {
        match self {
            Status::Pending => "Pending",
            Status::InProgress => "In progress",
            Status::Done => "Done",
        }
    }
}

/// Totals shown in the stat tiles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// All tasks.
    pub planned: usize,
    /// Number of modules.
    pub modules: usize,
    /// Tasks marked done.
    pub done: usize,
    /// Tasks in progress.
    pub in_progress: usize,
}

impl Roadmap {
    /// Totals across all modules.
    pub fn stats(&self) -> Stats {
        let tasks = self.modules.iter().flat_map(|m| &m.tasks);
        let mut stats = Stats { modules: self.modules.len(), ..Stats::default() };
        for task in tasks {
            stats.planned += 1;
            match task.status {
                Status::Done => stats.done += 1,
                Status::InProgress => stats.in_progress += 1,
                Status::Pending => {}
            }
        }
        stats
    }
}

impl RoadmapModule {
    /// Tasks marked done in this module.
    pub fn done(&self) -> usize {
        self.tasks.iter().filter(|t| t.status == Status::Done).count()
    }
}

/// Parse and validate roadmap JSON.
pub fn parse(json: &str) -> Result<Roadmap> {
    if json.len() > MAX_BYTES {
        return Err(Error::Invalid(format!("roadmap.json is larger than {MAX_BYTES} bytes")));
    }
    let roadmap: Roadmap =
        serde_json::from_str(json).map_err(|e| Error::Invalid(format!("roadmap.json: {e}")))?;
    validate(&roadmap)?;
    Ok(roadmap)
}

fn validate(roadmap: &Roadmap) -> Result<()> {
    let invalid = |why: &str| -> Result<()> { Err(Error::Invalid(format!("roadmap.json: {why}"))) };
    if roadmap.modules.is_empty() || roadmap.modules.len() > MAX_MODULES {
        return invalid("module count out of range");
    }
    if roadmap.stats().planned > MAX_TASKS {
        return invalid("too many tasks");
    }
    let too_long = |s: &str| s.chars().count() > MAX_TEXT;
    if too_long(&roadmap.phase) || too_long(&roadmap.updated) {
        return invalid("text field too long");
    }
    for module in &roadmap.modules {
        if module.key.is_empty() || too_long(&module.key) || too_long(&module.name) {
            return invalid("bad module key or name");
        }
        for task in &module.tasks {
            let assignee_bad = task.assignee.as_deref().is_some_and(too_long);
            if task.id.is_empty() || too_long(&task.id) || too_long(&task.name) || assignee_bad {
                return invalid("bad task id, name or assignee");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "phase": "Planning phase",
        "modules": [
            { "key": "organize", "name": "Organize", "tasks": [
                { "id": "merge-pdf", "name": "Merge PDF", "assignee": "Shafat", "status": "done" },
                { "id": "split-pdf", "name": "Split PDF", "status": "in_progress" },
                { "id": "remove-pages", "name": "Remove pages", "status": "weird" },
                { "id": "extract-pages", "name": "Extract pages" }
            ]}
        ]
    }"#;

    #[test]
    fn parses_and_counts() {
        let r = parse(SAMPLE).unwrap();
        let s = r.stats();
        assert_eq!((s.planned, s.modules, s.done, s.in_progress), (4, 1, 1, 1));
        assert_eq!(r.modules[0].done(), 1);
        assert_eq!(r.modules[0].tasks[0].assignee.as_deref(), Some("Shafat"));
    }

    #[test]
    fn unknown_or_missing_status_is_pending() {
        let r = parse(SAMPLE).unwrap();
        assert_eq!(r.modules[0].tasks[2].status, Status::Pending);
        assert_eq!(r.modules[0].tasks[3].status, Status::Pending);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse("not json").is_err());
        assert!(parse(r#"{"modules": []}"#).is_err());
        assert!(parse(&"x".repeat(MAX_BYTES + 1)).is_err());
        let long = format!(
            r#"{{"modules":[{{"key":"a","name":"{}","tasks":[]}}]}}"#,
            "n".repeat(MAX_TEXT + 1)
        );
        assert!(parse(&long).is_err());
    }
}
