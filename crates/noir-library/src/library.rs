//! Library service: the facade the UI talks to. All heavy work (scanning,
//! probing, persistence) happens off the calling thread where practical.

use crate::database::{Database, PlaylistRecord};
use crate::scanner::LibraryScanner;
use crossbeam_channel::{unbounded, Receiver};
use noir_core::errors::NoirError;
use noir_core::events::HistoryEntry;
use noir_core::models::MediaItem;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::warn;

/// High-level library operations. Cheap to clone and share.
#[derive(Clone)]
pub struct LibraryService {
    db: Arc<Mutex<Database>>,
    scanner: LibraryScanner,
}

impl LibraryService {
    pub fn open(db_path: &Path) -> Result<Self, NoirError> {
        let db = Database::open(db_path)?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            scanner: LibraryScanner::new(),
        })
    }

    /// In-memory database for tests.
    pub fn in_memory() -> Result<Self, NoirError> {
        let db = Database::open(Path::new(":memory:"))?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            scanner: LibraryScanner::new(),
        })
    }

    // ---- media ----

    pub fn add_items(&self, items: &[MediaItem]) -> Result<(), NoirError> {
        let db = self.db.lock().unwrap();
        for item in items {
            db.upsert_media(item)?;
        }
        Ok(())
    }

    pub fn all_media(&self) -> Vec<MediaItem> {
        self.with_db(|db| db.all_media()).unwrap_or_default()
    }

    pub fn recent_media(&self, limit: u32) -> Vec<MediaItem> {
        self.with_db(|db| db.recent_media(limit))
            .unwrap_or_default()
    }

    pub fn frequently_played(&self, limit: u32) -> Vec<MediaItem> {
        self.with_db(|db| db.frequently_played(limit))
            .unwrap_or_default()
    }

    pub fn favorites(&self) -> Vec<MediaItem> {
        self.with_db(|db| db.favorites()).unwrap_or_default()
    }

    pub fn search(&self, query: &str) -> Vec<MediaItem> {
        if query.trim().is_empty() {
            return self.all_media();
        }
        self.with_db(|db| db.search(query, 500)).unwrap_or_default()
    }

    pub fn set_favorite(&self, path: &Path, favorite: bool) {
        let db = self.db.lock().unwrap();
        if let Err(e) = db.set_favorite(path, favorite) {
            warn!("set_favorite failed: {e}");
        }
    }

    pub fn remove_missing(&self) -> u64 {
        self.with_db(|db| db.remove_missing()).unwrap_or(0)
    }

    // ---- scanning ----

    /// Register a directory and start a background scan. Found items are
    /// persisted as they arrive.
    pub fn scan_directory(&self, directory: PathBuf) -> Receiver<LibraryScanUpdate> {
        let (tx, rx) = unbounded();
        {
            let db = self.db.lock().unwrap();
            let _ = db.add_directory(&directory.to_string_lossy());
        }
        let (result_tx, result_rx) = unbounded::<Vec<MediaItem>>();
        self.scanner.scan(directory.clone(), result_tx);
        let db = self.db.clone();
        let dir_display = directory.to_string_lossy().to_string();
        std::thread::Builder::new()
            .name("noir-scan-persist".into())
            .spawn(move || {
                let mut total = 0u64;
                while let Ok(batch) = result_rx.recv() {
                    total += batch.len() as u64;
                    if tx
                        .send(LibraryScanUpdate::Found(batch.len() as u64))
                        .is_err()
                    {
                        return;
                    }
                    let db = db.lock().unwrap();
                    for item in &batch {
                        if let Err(e) = db.upsert_media(item) {
                            warn!("scan persist failed: {e}");
                        }
                    }
                    drop(db);
                    if tx.send(LibraryScanUpdate::Persisted).is_err() {
                        return;
                    }
                }
                let _ = tx.send(LibraryScanUpdate::Finished { found: total });
                let _ = dir_display;
            })
            .expect("failed to spawn scan persistence thread");
        rx
    }

    /// Progress events from the scanner.
    pub fn scanner(&self) -> &LibraryScanner {
        &self.scanner
    }

    pub fn directories(&self) -> Vec<String> {
        self.with_db(|db| db.directories()).unwrap_or_default()
    }

    pub fn remove_directory(&self, path: &str) {
        let db = self.db.lock().unwrap();
        let _ = db.remove_directory(path);
    }

    // ---- history ----

    pub fn record_history(&self, entry: &HistoryEntry) {
        let db = self.db.lock().unwrap();
        if let Err(e) = db.record_history(entry) {
            warn!("record_history failed: {e}");
        }
    }

    pub fn history(&self, limit: u32) -> Vec<HistoryEntry> {
        self.with_db(|db| db.history(limit)).unwrap_or_default()
    }

    pub fn resume_position(&self, path: &Path) -> Option<f64> {
        self.with_db(|db| db.resume_position(path)).unwrap_or(None)
    }

    pub fn clear_history(&self) {
        let db = self.db.lock().unwrap();
        let _ = db.clear_history();
    }

    // ---- playlists ----

    pub fn save_playlist(&self, name: &str, paths: &[String]) {
        let db = self.db.lock().unwrap();
        let _ = db.save_playlist(name, paths);
    }

    pub fn playlists(&self) -> Vec<PlaylistRecord> {
        self.with_db(|db| db.playlists()).unwrap_or_default()
    }

    pub fn delete_playlist(&self, id: i64) {
        let db = self.db.lock().unwrap();
        let _ = db.delete_playlist(id);
    }

    fn with_db<T>(&self, f: impl FnOnce(&Database) -> Result<T, NoirError>) -> Option<T> {
        let db = self.db.lock().unwrap();
        f(&db).ok()
    }
}

/// Progress updates for a running scan.
#[derive(Debug, Clone, PartialEq)]
pub enum LibraryScanUpdate {
    Found(u64),
    Persisted,
    Finished { found: u64 },
    Failed { directory: String, error: String },
}
