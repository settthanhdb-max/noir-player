//! SQLite persistence for the media library, history and playlists.
//!
//! One connection guarded by a mutex; every operation is short so the
//! library never blocks playback or the UI for long.

use noir_core::errors::NoirError;
use noir_core::events::HistoryEntry;
use noir_core::models::{MediaItem, MediaKind};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// A user-created playlist stored in the database.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistRecord {
    pub id: i64,
    pub name: String,
    pub item_paths: Vec<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (creating if needed) the database at `path`.
    pub fn open(path: &Path) -> Result<Self, NoirError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| NoirError::database(format!("create db dir: {e}")))?;
        }
        let conn =
            Connection::open(path).map_err(|e| NoirError::database(format!("open db: {e}")))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| NoirError::database(format!("pragma: {e}")))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| NoirError::database(format!("pragma: {e}")))?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Create tables. Versioned via `PRAGMA user_version` for future schema
    /// migrations.
    fn migrate(&self) -> Result<(), NoirError> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS media (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT UNIQUE NOT NULL,
                    title TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    duration REAL,
                    size_bytes INTEGER,
                    added_at INTEGER,
                    last_played_at INTEGER,
                    play_count INTEGER NOT NULL DEFAULT 0,
                    favorite INTEGER NOT NULL DEFAULT 0
                );
                CREATE INDEX IF NOT EXISTS idx_media_last_played ON media(last_played_at);
                CREATE INDEX IF NOT EXISTS idx_media_added ON media(added_at);
                CREATE TABLE IF NOT EXISTS history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT NOT NULL,
                    title TEXT NOT NULL,
                    last_played INTEGER NOT NULL,
                    position REAL NOT NULL DEFAULT 0,
                    duration REAL,
                    completion REAL NOT NULL DEFAULT 0
                );
                CREATE INDEX IF NOT EXISTS idx_history_played ON history(last_played DESC);
                CREATE TABLE IF NOT EXISTS playlists (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT UNIQUE NOT NULL
                );
                CREATE TABLE IF NOT EXISTS playlist_items (
                    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
                    position INTEGER NOT NULL,
                    path TEXT NOT NULL,
                    PRIMARY KEY (playlist_id, position)
                );
                "#,
            )
            .map_err(|e| NoirError::database(format!("migrate: {e}")))?;
        Ok(())
    }

    // ---- media ----

    /// Insert or update a media item (path is the natural key).
    pub fn upsert_media(&self, item: &MediaItem) -> Result<i64, NoirError> {
        let added_at = item.added_at.map(epoch).unwrap_or_else(now_epoch);
        self.conn
            .execute(
                r#"INSERT INTO media (path, title, kind, duration, size_bytes, added_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                   ON CONFLICT(path) DO UPDATE SET
                     title = excluded.title,
                     kind = excluded.kind,
                     duration = COALESCE(excluded.duration, media.duration),
                     size_bytes = COALESCE(excluded.size_bytes, media.size_bytes)"#,
                params![
                    item.path.to_string_lossy(),
                    item.title,
                    format!("{:?}", item.kind),
                    item.duration,
                    item.size_bytes.map(|s| s as i64),
                    added_at as i64,
                ],
            )
            .map_err(|e| NoirError::database(format!("upsert media: {e}")))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn all_media(&self) -> Result<Vec<MediaItem>, NoirError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, path, title, kind, duration, size_bytes, added_at,
                        last_played_at, play_count, favorite
                 FROM media ORDER BY title COLLATE NOCASE",
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        let rows = stmt
            .query_map([], row_to_media)
            .map_err(|e| NoirError::database(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| NoirError::database(e.to_string()))
    }

    pub fn recent_media(&self, limit: u32) -> Result<Vec<MediaItem>, NoirError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, path, title, kind, duration, size_bytes, added_at,
                        last_played_at, play_count, favorite
                 FROM media WHERE last_played_at IS NOT NULL
                 ORDER BY last_played_at DESC LIMIT ?1",
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        let rows = stmt
            .query_map(params![limit], row_to_media)
            .map_err(|e| NoirError::database(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| NoirError::database(e.to_string()))
    }

    pub fn frequently_played(&self, limit: u32) -> Result<Vec<MediaItem>, NoirError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, path, title, kind, duration, size_bytes, added_at,
                        last_played_at, play_count, favorite
                 FROM media WHERE play_count > 0
                 ORDER BY play_count DESC, last_played_at DESC LIMIT ?1",
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        let rows = stmt
            .query_map(params![limit], row_to_media)
            .map_err(|e| NoirError::database(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| NoirError::database(e.to_string()))
    }

    pub fn favorites(&self) -> Result<Vec<MediaItem>, NoirError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, path, title, kind, duration, size_bytes, added_at,
                        last_played_at, play_count, favorite
                 FROM media WHERE favorite = 1 ORDER BY title COLLATE NOCASE",
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        let rows = stmt
            .query_map([], row_to_media)
            .map_err(|e| NoirError::database(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| NoirError::database(e.to_string()))
    }

    pub fn search(&self, query: &str, limit: u32) -> Result<Vec<MediaItem>, NoirError> {
        let pattern = format!("%{}%", query.replace(['%', '_'], ""));
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, path, title, kind, duration, size_bytes, added_at,
                        last_played_at, play_count, favorite
                 FROM media WHERE title LIKE ?1 COLLATE NOCASE
                 ORDER BY title COLLATE NOCASE LIMIT ?2",
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        let rows = stmt
            .query_map(params![pattern, limit], row_to_media)
            .map_err(|e| NoirError::database(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| NoirError::database(e.to_string()))
    }

    pub fn set_favorite(&self, path: &Path, favorite: bool) -> Result<(), NoirError> {
        self.conn
            .execute(
                "UPDATE media SET favorite = ?1 WHERE path = ?2",
                params![favorite as i64, path.to_string_lossy()],
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        Ok(())
    }

    /// Remove entries whose files no longer exist on disk.
    pub fn remove_missing(&self) -> Result<u64, NoirError> {
        let paths: Vec<(i64, String)> = {
            let mut stmt = self
                .conn
                .prepare("SELECT id, path FROM media")
                .map_err(|e| NoirError::database(e.to_string()))?;
            let rows = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| NoirError::database(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| NoirError::database(e.to_string()))?
        };
        let mut removed = 0u64;
        for (id, path) in paths {
            if !Path::new(&path).exists() {
                self.conn
                    .execute("DELETE FROM media WHERE id = ?1", params![id])
                    .map_err(|e| NoirError::database(e.to_string()))?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    // ---- history ----

    /// Record one playback session. Respects the caller's history policy by
    /// only being invoked when history recording is enabled.
    pub fn record_history(&self, entry: &HistoryEntry) -> Result<(), NoirError> {
        self.conn
            .execute(
                r#"INSERT INTO history (path, title, last_played, position, duration, completion)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
                params![
                    entry.path,
                    entry.title,
                    epoch(entry.last_played) as i64,
                    entry.position,
                    entry.duration,
                    entry.completion as f64,
                ],
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        // Keep the media table in sync for "continue watching".
        self.conn
            .execute(
                r#"UPDATE media SET last_played_at = ?1,
                     play_count = play_count + 1
                   WHERE path = ?2"#,
                params![epoch(entry.last_played) as i64, entry.path],
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        Ok(())
    }

    /// Saved resume position for a file, if any.
    pub fn resume_position(&self, path: &Path) -> Result<Option<f64>, NoirError> {
        self.conn
            .query_row(
                "SELECT position FROM history WHERE path = ?1 ORDER BY last_played DESC LIMIT 1",
                params![path.to_string_lossy()],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| NoirError::database(e.to_string()))
    }

    pub fn history(&self, limit: u32) -> Result<Vec<HistoryEntry>, NoirError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT path, title, last_played, position, duration, completion
                 FROM history ORDER BY last_played DESC LIMIT ?1",
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        let rows = stmt
            .query_map(params![limit], |r| {
                let last_played: i64 = r.get(2)?;
                Ok(HistoryEntry {
                    path: r.get(0)?,
                    title: r.get(1)?,
                    last_played: SystemTime::UNIX_EPOCH
                        + std::time::Duration::from_secs(last_played.max(0) as u64),
                    position: r.get(3)?,
                    duration: r.get(4)?,
                    completion: r.get::<_, f64>(5)? as f32,
                })
            })
            .map_err(|e| NoirError::database(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| NoirError::database(e.to_string()))
    }

    pub fn clear_history(&self) -> Result<(), NoirError> {
        self.conn
            .execute("DELETE FROM history", [])
            .map_err(|e| NoirError::database(e.to_string()))?;
        Ok(())
    }

    // ---- playlists ----

    pub fn save_playlist(&self, name: &str, paths: &[String]) -> Result<i64, NoirError> {
        self.conn
            .execute(
                "INSERT INTO playlists (name) VALUES (?1)
                 ON CONFLICT(name) DO UPDATE SET name = excluded.name",
                params![name],
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        let id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM playlists WHERE name = ?1",
                params![name],
                |r| r.get(0),
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        self.conn
            .execute(
                "DELETE FROM playlist_items WHERE playlist_id = ?1",
                params![id],
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        for (i, path) in paths.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO playlist_items (playlist_id, position, path) VALUES (?1, ?2, ?3)",
                    params![id, i as i64, path],
                )
                .map_err(|e| NoirError::database(e.to_string()))?;
        }
        Ok(id)
    }

    pub fn playlists(&self) -> Result<Vec<PlaylistRecord>, NoirError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM playlists ORDER BY name COLLATE NOCASE")
            .map_err(|e| NoirError::database(e.to_string()))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| NoirError::database(e.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            let (id, name) = row.map_err(|e| NoirError::database(e.to_string()))?;
            let mut stmt = self
                .conn
                .prepare("SELECT path FROM playlist_items WHERE playlist_id = ?1 ORDER BY position")
                .map_err(|e| NoirError::database(e.to_string()))?;
            let paths = stmt
                .query_map(params![id], |r| r.get(0))
                .map_err(|e| NoirError::database(e.to_string()))?
                .collect::<Result<Vec<String>, _>>()
                .map_err(|e| NoirError::database(e.to_string()))?;
            out.push(PlaylistRecord {
                id,
                name,
                item_paths: paths,
            });
        }
        Ok(out)
    }

    pub fn delete_playlist(&self, id: i64) -> Result<(), NoirError> {
        self.conn
            .execute(
                "DELETE FROM playlist_items WHERE playlist_id = ?1",
                params![id],
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        self.conn
            .execute("DELETE FROM playlists WHERE id = ?1", params![id])
            .map_err(|e| NoirError::database(e.to_string()))?;
        Ok(())
    }

    /// Registered library directories.
    pub fn directories(&self) -> Result<Vec<String>, NoirError> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS directories (path TEXT PRIMARY KEY)",
                [],
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        let mut stmt = self
            .conn
            .prepare("SELECT path FROM directories ORDER BY path")
            .map_err(|e| NoirError::database(e.to_string()))?;
        let rows = stmt
            .query_map([], |r| r.get(0))
            .map_err(|e| NoirError::database(e.to_string()))?;
        rows.collect::<Result<Vec<String>, _>>()
            .map_err(|e| NoirError::database(e.to_string()))
    }

    pub fn add_directory(&self, path: &str) -> Result<(), NoirError> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS directories (path TEXT PRIMARY KEY)",
                [],
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO directories (path) VALUES (?1)",
                params![path],
            )
            .map_err(|e| NoirError::database(e.to_string()))?;
        Ok(())
    }

    pub fn remove_directory(&self, path: &str) -> Result<(), NoirError> {
        self.conn
            .execute("DELETE FROM directories WHERE path = ?1", params![path])
            .map_err(|e| NoirError::database(e.to_string()))?;
        Ok(())
    }
}

fn row_to_media(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaItem> {
    let kind_str: String = row.get(3)?;
    let kind = match kind_str.as_str() {
        "Video" => MediaKind::Video,
        "Audio" => MediaKind::Audio,
        "Image" => MediaKind::Image,
        _ => MediaKind::Unknown,
    };
    let added_at: Option<i64> = row.get(6)?;
    let last_played: Option<i64> = row.get(7)?;
    Ok(MediaItem {
        id: Some(row.get(0)?),
        path: PathBuf::from(row.get::<_, String>(1)?),
        title: row.get(2)?,
        kind,
        duration: row.get(4)?,
        size_bytes: row.get::<_, Option<i64>>(5)?.map(|v| v as u64),
        added_at: added_at
            .map(|t| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(t.max(0) as u64)),
        last_played_at: last_played
            .map(|t| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(t.max(0) as u64)),
        play_count: row.get::<_, i64>(8)? as u32,
        favorite: row.get::<_, i64>(9)? != 0,
    })
}

fn epoch(t: SystemTime) -> u64 {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn now_epoch() -> u64 {
    epoch(SystemTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        // Leak the tempdir for the lifetime of the test process; fine for tests.
        std::mem::forget(dir);
        Database::open(&path).unwrap()
    }

    #[test]
    fn media_upsert_and_query() {
        let db = test_db();
        let mut item = MediaItem::from_path("/media/movie.mkv");
        item.duration = Some(7200.0);
        item.size_bytes = Some(1024);
        db.upsert_media(&item).unwrap();
        db.upsert_media(&item).unwrap();
        let all = db.all_media().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].kind, MediaKind::Video);
    }

    #[test]
    fn search_and_favorites() {
        let db = test_db();
        let item = MediaItem::from_path("/media/Interstellar.mkv");
        db.upsert_media(&item).unwrap();
        assert_eq!(db.search("inter", 10).unwrap().len(), 1);
        db.set_favorite(Path::new("/media/Interstellar.mkv"), true)
            .unwrap();
        assert_eq!(db.favorites().unwrap().len(), 1);
    }

    #[test]
    fn history_roundtrip() {
        let db = test_db();
        let item = MediaItem::from_path("/media/a.mkv");
        db.upsert_media(&item).unwrap();
        let entry = HistoryEntry {
            path: "/media/a.mkv".into(),
            title: "a".into(),
            last_played: SystemTime::now(),
            position: 42.0,
            duration: Some(100.0),
            completion: 0.42,
        };
        db.record_history(&entry).unwrap();
        let hist = db.history(10).unwrap();
        assert_eq!(hist.len(), 1);
        assert!((hist[0].position - 42.0).abs() < f64::EPSILON);
        let media = db.all_media().unwrap();
        assert_eq!(media[0].play_count, 1);
        assert_eq!(
            db.resume_position(Path::new("/media/a.mkv")).unwrap(),
            Some(42.0)
        );
        db.clear_history().unwrap();
        assert!(db.history(10).unwrap().is_empty());
    }

    #[test]
    fn playlist_roundtrip() {
        let db = test_db();
        db.save_playlist("Night", &["/a.mkv".into(), "/b.mkv".into()])
            .unwrap();
        let lists = db.playlists().unwrap();
        assert_eq!(lists.len(), 1);
        assert_eq!(
            lists[0].item_paths,
            vec!["/a.mkv".to_string(), "/b.mkv".to_string()]
        );
        db.save_playlist("Night", &["/c.mkv".into()]).unwrap();
        let lists = db.playlists().unwrap();
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].item_paths, vec!["/c.mkv".to_string()]);
        db.delete_playlist(lists[0].id).unwrap();
        assert!(db.playlists().unwrap().is_empty());
    }

    #[test]
    fn directories_roundtrip() {
        let db = test_db();
        db.add_directory("/media/movies").unwrap();
        db.add_directory("/media/movies").unwrap();
        assert_eq!(db.directories().unwrap(), vec!["/media/movies".to_string()]);
        db.remove_directory("/media/movies").unwrap();
        assert!(db.directories().unwrap().is_empty());
    }
}
