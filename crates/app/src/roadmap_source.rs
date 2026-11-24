//! Where the roadmap comes from: GitHub (live) → SQLite cache → built-in copy.
//!
//! Everything here blocks (network, DB). Call it through `jobs::blocking`, never
//! on the UI thread.

use std::io::Read;
use std::sync::Arc;
use std::time::Duration;

use dk_core::roadmap::{self, Roadmap, MAX_BYTES};
use dk_core::store::Store;
use dk_core::{Error, Result};

/// Raw file on the default branch. Edit roadmap.json on GitHub and every app picks it up.
pub const RAW_URL: &str =
    "https://raw.githubusercontent.com/Shafat21/DocumentKit/main/roadmap.json";
/// Human-friendly page for the same file.
pub const FILE_URL: &str = "https://github.com/Shafat21/DocumentKit/blob/main/roadmap.json";

const TIMEOUT: Duration = Duration::from_secs(10);

/// Compiled-in copy so the screen works on first launch with no network.
static BUNDLED_JSON: &str = include_str!("../../../roadmap.json");

/// Where the data on screen came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Bundled,
    Cached { fetched_at: String },
    Live { fetched_at: String },
}

/// A roadmap ready to render.
#[derive(Debug, Clone)]
pub struct Loaded {
    pub roadmap: Arc<Roadmap>,
    pub origin: Origin,
}

/// Built-in copy. Validated by a test, so the fallback branch is only defensive.
pub fn bundled() -> Loaded {
    let roadmap = roadmap::parse(BUNDLED_JSON).unwrap_or_else(|e| {
        tracing::error!("bundled roadmap.json is invalid: {e}");
        Roadmap::default()
    });
    Loaded {
        roadmap: Arc::new(roadmap),
        origin: Origin::Bundled,
    }
}

/// Last copy saved from GitHub, if it exists and still parses.
pub fn cached(store: &Store) -> Option<Loaded> {
    let cached = match store.roadmap_cache() {
        Ok(Some(c)) => c,
        Ok(None) => return None,
        Err(e) => {
            tracing::warn!("roadmap cache unreadable: {e}");
            return None;
        }
    };
    match roadmap::parse(&cached.json) {
        Ok(r) => Some(Loaded {
            roadmap: Arc::new(r),
            origin: Origin::Cached {
                fetched_at: cached.fetched_at,
            },
        }),
        Err(e) => {
            tracing::warn!("roadmap cache invalid, ignoring: {e}");
            None
        }
    }
}

/// Download, validate, and save. Only valid JSON ever reaches the cache.
pub fn fetch_and_cache(store: &Store) -> Result<Loaded> {
    let json = download()?;
    let roadmap = roadmap::parse(&json)?;
    let fetched_at = store.save_roadmap_cache(&json)?;
    Ok(Loaded {
        roadmap: Arc::new(roadmap),
        origin: Origin::Live { fetched_at },
    })
}

fn download() -> Result<String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(TIMEOUT)
        .https_only(true)
        .user_agent(concat!("DocumentKit/", env!("CARGO_PKG_VERSION")))
        .build();
    let response = agent
        .get(RAW_URL)
        .call()
        .map_err(|e| Error::Network(e.to_string()))?;

    // Read at most MAX_BYTES + 1 so an oversized (or hostile) response can't exhaust memory.
    let mut body = String::new();
    response
        .into_reader()
        .take(MAX_BYTES as u64 + 1)
        .read_to_string(&mut body)
        .map_err(|e| Error::Network(e.to_string()))?;
    if body.len() > MAX_BYTES {
        return Err(Error::Invalid("roadmap.json response too large".into()));
    }
    Ok(body)
}

/// "2026-11-20T09:15:00Z" → "2026-11-20 09:15 UTC".
pub fn format_timestamp(iso: &str) -> String {
    match (iso.get(..10), iso.get(11..16)) {
        (Some(date), Some(time)) => format!("{date} {time} UTC"),
        _ => iso.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn bundled_roadmap_is_valid() {
        let parsed = roadmap::parse(BUNDLED_JSON).expect("roadmap.json must parse");
        assert_eq!(parsed.stats().modules, dk_core::ModuleId::ALL.len());
    }

    /// roadmap.json and the catalog must list the same tools, so the two never drift.
    #[test]
    fn bundled_roadmap_matches_catalog() {
        let parsed = roadmap::parse(BUNDLED_JSON).unwrap();
        let roadmap_ids: HashSet<_> = parsed
            .modules
            .iter()
            .flat_map(|m| &m.tasks)
            .map(|t| t.id.as_str())
            .collect();
        let catalog_ids: HashSet<_> = dk_core::catalog::TOOLS.iter().map(|t| t.id).collect();
        assert_eq!(roadmap_ids, catalog_ids);

        let module_keys: Vec<_> = parsed.modules.iter().map(|m| m.key.as_str()).collect();
        let expected: Vec<_> = dk_core::ModuleId::ALL.iter().map(|m| m.key()).collect();
        assert_eq!(module_keys, expected);
    }

    #[test]
    fn cache_round_trip_and_invalid_cache_ignored() {
        let store = Store::in_memory().unwrap();
        assert!(cached(&store).is_none());
        store.save_roadmap_cache(BUNDLED_JSON).unwrap();
        assert!(matches!(
            cached(&store).unwrap().origin,
            Origin::Cached { .. }
        ));
        store.save_roadmap_cache("garbage").unwrap();
        assert!(cached(&store).is_none());
    }

    #[test]
    fn formats_timestamp() {
        assert_eq!(
            format_timestamp("2026-11-20T09:15:00Z"),
            "2026-11-20 09:15 UTC"
        );
        assert_eq!(format_timestamp("short"), "short");
    }
}
