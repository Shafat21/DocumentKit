//! Local SQLite store: settings and recently opened documents.
//!
//! `rusqlite::Connection` is not `Sync`, so it lives behind a `Mutex`; that makes
//! `Store` `Send + Sync` and safe to share as `Arc<Store>`. Every query is short
//! and parameterised. Call store methods off the UI thread for anything that could
//! grow (the recent list is capped by `limit`).

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension};

use crate::{Error, Result};

/// Schema migrations, applied in order. Index + 1 = `PRAGMA user_version`.
/// Never edit a shipped migration; append a new one.
const MIGRATIONS: &[&str] = &[
    // v1
    "CREATE TABLE settings (
        key   TEXT PRIMARY KEY NOT NULL,
        value TEXT NOT NULL
    );
    CREATE TABLE recent_documents (
        id         INTEGER PRIMARY KEY,
        path       TEXT NOT NULL UNIQUE,
        name       TEXT NOT NULL,
        pages      INTEGER NOT NULL,
        size_bytes INTEGER NOT NULL,
        opened_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
    );
    CREATE INDEX recent_documents_opened_at ON recent_documents (opened_at DESC);",
    // v2: last good roadmap.json from GitHub, shown when offline.
    "CREATE TABLE roadmap_cache (
        id         INTEGER PRIMARY KEY CHECK (id = 1),
        json       TEXT NOT NULL,
        fetched_at TEXT NOT NULL
    );",
];

/// Cached roadmap.json and when it was fetched (UTC, ISO-8601).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedRoadmap {
    /// Raw JSON as downloaded (already validated before it was saved).
    pub json: String,
    /// When it was downloaded.
    pub fetched_at: String,
}

/// A recently opened document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentDoc {
    /// Absolute path on disk.
    pub path: String,
    /// File name shown in the list.
    pub name: String,
    /// Page count.
    pub pages: u32,
    /// File size in bytes.
    pub size_bytes: u64,
    /// UTC timestamp, ISO-8601.
    pub opened_at: String,
}

/// Handle to the local database.
#[derive(Debug)]
pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// Open (or create) the database file and apply pending migrations.
    pub fn open(path: &Path) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    /// In-memory database, for tests.
    pub fn in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        // journal_mode returns a row, so it needs the _and_check variant.
        conn.pragma_update_and_check(None, "journal_mode", "WAL", |_| Ok(()))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| Error::Internal("store mutex poisoned".into()))
    }

    /// Read a setting.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn()?;
        let value = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .optional()?;
        Ok(value)
    }

    /// Write a setting (insert or replace).
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn()?.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Most recently opened documents, newest first.
    pub fn recent_documents(&self, limit: u32) -> Result<Vec<RecentDoc>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare_cached(
            "SELECT path, name, pages, size_bytes, opened_at
             FROM recent_documents ORDER BY opened_at DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |r| {
            Ok(RecentDoc {
                path: r.get(0)?,
                name: r.get(1)?,
                pages: r.get(2)?,
                size_bytes: r.get(3)?,
                opened_at: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Record that a document was opened (moves it to the top if already present).
    pub fn touch_recent(&self, path: &str, name: &str, pages: u32, size_bytes: u64) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO recent_documents (path, name, pages, size_bytes)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                pages = excluded.pages,
                size_bytes = excluded.size_bytes,
                opened_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
            params![path, name, pages, size_bytes],
        )?;
        prune_recent(&conn, MAX_RECENT)
    }

    /// Forget one document.
    pub fn remove_recent(&self, path: &str) -> Result<()> {
        self.conn()?.execute(
            "DELETE FROM recent_documents WHERE path = ?1",
            params![path],
        )?;
        Ok(())
    }

    /// Forget every document.
    pub fn clear_recent(&self) -> Result<()> {
        self.conn()?.execute("DELETE FROM recent_documents", [])?;
        Ok(())
    }
}

/// Most documents kept in the recent list; older ones are pruned on insert.
pub const MAX_RECENT: u32 = 500;

fn prune_recent(conn: &Connection, keep: u32) -> Result<()> {
    conn.execute(
        "DELETE FROM recent_documents WHERE id NOT IN (
            SELECT id FROM recent_documents ORDER BY opened_at DESC, id DESC LIMIT ?1
         )",
        params![keep],
    )?;
    Ok(())
}

impl Store {
    /// Last saved roadmap, if any.
    pub fn roadmap_cache(&self) -> Result<Option<CachedRoadmap>> {
        let conn = self.conn()?;
        let cached = conn
            .query_row(
                "SELECT json, fetched_at FROM roadmap_cache WHERE id = 1",
                [],
                |r| {
                    Ok(CachedRoadmap {
                        json: r.get(0)?,
                        fetched_at: r.get(1)?,
                    })
                },
            )
            .optional()?;
        Ok(cached)
    }

    /// Replace the cached roadmap; returns the stored `fetched_at`.
    pub fn save_roadmap_cache(&self, json: &str) -> Result<String> {
        let fetched_at = self.conn()?.query_row(
            "INSERT INTO roadmap_cache (id, json, fetched_at)
             VALUES (1, ?1, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
             ON CONFLICT(id) DO UPDATE SET json = excluded.json, fetched_at = excluded.fetched_at
             RETURNING fetched_at",
            params![json],
            |r| r.get(0),
        )?;
        Ok(fetched_at)
    }
}

fn migrate(conn: &Connection) -> Result<()> {
    let current: usize = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", i + 1)?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_round_trip() {
        let store = Store::in_memory().unwrap();
        assert_eq!(store.get_setting("theme").unwrap(), None);
        store.set_setting("theme", "dark").unwrap();
        store.set_setting("theme", "light").unwrap();
        assert_eq!(
            store.get_setting("theme").unwrap().as_deref(),
            Some("light")
        );
    }

    #[test]
    fn migrate_is_idempotent() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn().unwrap();
        migrate(&conn).unwrap();
        let v: usize = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, MIGRATIONS.len());
    }

    #[test]
    fn recent_documents_newest_first_and_limited() {
        let store = Store::in_memory().unwrap();
        store.touch_recent("/a.pdf", "a.pdf", 1, 10).unwrap();
        store.touch_recent("/b.pdf", "b.pdf", 2, 20).unwrap();
        store.touch_recent("/c.pdf", "c.pdf", 3, 30).unwrap();
        let docs = store.recent_documents(2).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].name, "c.pdf");
        assert_eq!(docs[1].name, "b.pdf");
    }

    #[test]
    fn roadmap_cache_round_trip() {
        let store = Store::in_memory().unwrap();
        assert_eq!(store.roadmap_cache().unwrap(), None);
        store.save_roadmap_cache("{\"a\":1}").unwrap();
        let at = store.save_roadmap_cache("{\"a\":2}").unwrap();
        let cached = store.roadmap_cache().unwrap().unwrap();
        assert_eq!(cached.json, "{\"a\":2}");
        assert_eq!(cached.fetched_at, at);
    }

    #[test]
    fn remove_and_clear_recent() {
        let store = Store::in_memory().unwrap();
        store.touch_recent("/a.pdf", "a.pdf", 1, 10).unwrap();
        store.touch_recent("/b.pdf", "b.pdf", 1, 10).unwrap();
        store.remove_recent("/a.pdf").unwrap();
        let docs = store.recent_documents(10).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name, "b.pdf");
        store.clear_recent().unwrap();
        assert!(store.recent_documents(10).unwrap().is_empty());
    }

    #[test]
    fn prune_keeps_newest() {
        let store = Store::in_memory().unwrap();
        for i in 0..5 {
            store.touch_recent(&format!("/{i}.pdf"), "x", 1, 1).unwrap();
        }
        prune_recent(&store.conn().unwrap(), 2).unwrap();
        let paths: Vec<_> = store
            .recent_documents(10)
            .unwrap()
            .into_iter()
            .map(|d| d.path)
            .collect();
        assert_eq!(paths, ["/4.pdf", "/3.pdf"]);
    }

    #[test]
    fn touching_existing_doc_does_not_duplicate() {
        let store = Store::in_memory().unwrap();
        store.touch_recent("/a.pdf", "a.pdf", 1, 10).unwrap();
        store.touch_recent("/a.pdf", "a.pdf", 5, 50).unwrap();
        let docs = store.recent_documents(10).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].pages, 5);
    }
}
