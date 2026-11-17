//! Background directory scanner.
//!
//! Scanning runs on a worker thread and reports progress through a channel;
//! the UI stays responsive and receives `LibraryEvent`s.

use crossbeam_channel::{unbounded, Receiver, Sender};
use noir_core::events::LibraryEvent;
use noir_core::models::MediaItem;
#[cfg(test)]
use noir_core::models::MediaKind;
use std::path::{Path, PathBuf};
use std::thread;
use tracing::{info, warn};
use walkdir::WalkDir;

/// Extensions considered media. Support is determined by the playback
/// backend at open time; this list only decides what the scanner indexes.
pub const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "mpg", "mpeg", "ts", "m2ts", "m4v", "ogv",
    "3gp", "mp3", "aac", "flac", "wav", "ogg", "opus", "m4a", "wma", "ape",
];

/// Handle to a running scanner. Drop-safe: a scan outlives the UI request.
#[derive(Clone)]
pub struct LibraryScanner {
    progress_tx: Sender<LibraryEvent>,
    progress_rx: Receiver<LibraryEvent>,
}

impl Default for LibraryScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryScanner {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            progress_tx: tx,
            progress_rx: rx,
        }
    }

    /// Subscribe to scanner progress events.
    pub fn subscribe(&self) -> Receiver<LibraryEvent> {
        self.progress_rx.clone()
    }

    /// Scan one directory on a background thread. Found items are sent to
    /// `result_tx` in batches; the caller persists them.
    pub fn scan(&self, directory: PathBuf, result_tx: Sender<Vec<MediaItem>>) {
        let progress_tx = self.progress_tx.clone();
        thread::Builder::new()
            .name("noir-scanner".into())
            .spawn(move || {
                let dir_display = directory.to_string_lossy().to_string();
                let _ = progress_tx.send(LibraryEvent::ScanStarted {
                    directory: dir_display.clone(),
                });
                let mut found = 0u64;
                let mut scanned = 0u64;
                let mut batch: Vec<MediaItem> = Vec::with_capacity(256);
                for entry in WalkDir::new(&directory)
                    .follow_links(false)
                    .into_iter()
                    .filter_entry(|e| e.depth() == 0 || !is_hidden(e))
                {
                    match entry {
                        Ok(entry) => {
                            scanned += 1;
                            if entry.file_type().is_file() {
                                if let Some(item) = media_item_from_file(entry.path()) {
                                    found += 1;
                                    batch.push(item);
                                    if batch.len() >= 256
                                        && result_tx.send(std::mem::take(&mut batch)).is_err()
                                    {
                                        return;
                                    }
                                }
                            }
                            if scanned % 512 == 0 {
                                let _ =
                                    progress_tx.send(LibraryEvent::ScanProgress { scanned, found });
                            }
                        }
                        Err(e) => {
                            // Permission errors and unreadable entries are
                            // logged and skipped, never fatal.
                            warn!(
                                "scan: skipping {}: {e}",
                                e.path()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_default()
                            );
                        }
                    }
                }
                if !batch.is_empty() {
                    let _ = result_tx.send(batch);
                }
                info!("scan of {dir_display} finished: {found} media files");
                let _ = progress_tx.send(LibraryEvent::ScanFinished {
                    directory: dir_display,
                    found,
                });
            })
            .expect("failed to spawn scanner thread");
    }

    /// Synchronous variant for tests and small directories.
    pub fn scan_blocking(&self, directory: &Path) -> Vec<MediaItem> {
        WalkDir::new(directory)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| e.depth() == 0 || !is_hidden(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| media_item_from_file(e.path()))
            .collect()
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') || s == "$RECYCLE.BIN" || s == "System Volume Information")
        .unwrap_or(false)
}

/// Build a `MediaItem` from a file path if the extension is a known media
/// type. Returns `None` for non-media files.
pub fn media_item_from_file(path: &Path) -> Option<MediaItem> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if !MEDIA_EXTENSIONS.contains(&ext.as_str()) {
        return None;
    }
    let mut item = MediaItem::from_path(path);
    if let Ok(meta) = std::fs::metadata(path) {
        item.size_bytes = Some(meta.len());
        item.added_at = meta.created().ok();
    }
    Some(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_media_files() {
        assert!(media_item_from_file(Path::new("/x/movie.mkv")).is_some());
        assert!(media_item_from_file(Path::new("/x/song.flac")).is_some());
        assert!(media_item_from_file(Path::new("/x/notes.txt")).is_none());
        assert_eq!(
            media_item_from_file(Path::new("/x/song.FLAC"))
                .unwrap()
                .kind,
            MediaKind::Audio
        );
    }

    #[test]
    fn scans_directory_tree() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.mkv"), b"x").unwrap();
        std::fs::write(dir.path().join("b.mp3"), b"x").unwrap();
        std::fs::write(dir.path().join(".hidden.mp4"), b"x").unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".git/c.mkv"), b"x").unwrap();
        std::fs::write(dir.path().join("readme.txt"), b"x").unwrap();
        let scanner = LibraryScanner::new();
        let items = scanner.scan_blocking(dir.path());
        let mut names: Vec<String> = items
            .iter()
            .map(|i| i.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        names.sort();
        assert_eq!(names, vec!["a.mkv".to_string(), "b.mp3".to_string()]);
    }
}
