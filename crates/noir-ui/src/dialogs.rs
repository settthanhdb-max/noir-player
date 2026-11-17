//! Non-blocking native file dialogs.
//!
//! `rfd` dialogs are modal and blocking, so they run on a worker thread and
//! results are polled once per frame. The UI thread never blocks.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub enum DialogResult {
    Paths(Vec<PathBuf>),
    Directory(PathBuf),
    Save(PathBuf),
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    OpenMedia,
    OpenSubtitle,
    PickDirectory,
    SaveM3u,
    OpenM3u,
    SaveJson,
    OpenJson,
}

struct InFlight {
    kind: DialogKind,
    context: Option<String>,
}

#[derive(Default)]
pub struct FileDialogs {
    rx: Option<Receiver<DialogResult>>,
    in_flight: Option<InFlight>,
}

impl FileDialogs {
    pub fn is_busy(&self) -> bool {
        self.in_flight.is_some()
    }

    pub fn busy_context(&self) -> Option<&str> {
        self.in_flight.as_ref().and_then(|f| f.context.as_deref())
    }

    pub fn in_flight_kind(&self) -> Option<DialogKind> {
        self.in_flight.as_ref().map(|f| f.kind)
    }

    /// Launch a dialog on a worker thread. Ignored while one is already open.
    pub fn request(&mut self, kind: DialogKind, context: Option<String>) {
        if self.in_flight.is_some() {
            return;
        }
        let (tx, rx): (Sender<DialogResult>, Receiver<DialogResult>) = channel();
        self.rx = Some(rx);
        self.in_flight = Some(InFlight { kind, context });
        std::thread::Builder::new()
            .name("noir-dialog".into())
            .spawn(move || {
                let result = run_dialog(kind, tx.clone());
                let _ = tx.send(result);
            })
            .expect("failed to spawn dialog thread");
    }

    /// Poll for a finished dialog: `(kind, context, result)`.
    pub fn poll(&mut self) -> Option<(DialogKind, Option<String>, DialogResult)> {
        let rx = self.rx.as_ref()?;
        match rx.try_recv() {
            Ok(result) => {
                let flight = self.in_flight.take()?;
                self.rx = None;
                Some((flight.kind, flight.context, result))
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.in_flight = None;
                self.rx = None;
                None
            }
        }
    }
}

fn run_dialog(kind: DialogKind, _tx: Sender<DialogResult>) -> DialogResult {
    match kind {
        DialogKind::OpenMedia => {
            let files = rfd::FileDialog::new()
                .add_filter("Media files", MEDIA_EXTENSIONS)
                .pick_files();
            files
                .map(DialogResult::Paths)
                .unwrap_or(DialogResult::Cancelled)
        }
        DialogKind::OpenSubtitle => rfd::FileDialog::new()
            .add_filter(
                "Subtitles",
                &["srt", "ass", "ssa", "vtt", "sub", "idx", "sup"],
            )
            .pick_file()
            .map(|f| DialogResult::Paths(vec![f]))
            .unwrap_or(DialogResult::Cancelled),
        DialogKind::PickDirectory => rfd::FileDialog::new()
            .pick_folder()
            .map(DialogResult::Directory)
            .unwrap_or(DialogResult::Cancelled),
        DialogKind::SaveM3u => rfd::FileDialog::new()
            .add_filter("M3U playlists", &["m3u", "m3u8"])
            .save_file()
            .map(DialogResult::Save)
            .unwrap_or(DialogResult::Cancelled),
        DialogKind::OpenM3u => rfd::FileDialog::new()
            .add_filter("M3U playlists", &["m3u", "m3u8"])
            .pick_file()
            .map(|f| DialogResult::Paths(vec![f]))
            .unwrap_or(DialogResult::Cancelled),
        DialogKind::SaveJson => rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .save_file()
            .map(DialogResult::Save)
            .unwrap_or(DialogResult::Cancelled),
        DialogKind::OpenJson => rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
            .map(|f| DialogResult::Paths(vec![f]))
            .unwrap_or(DialogResult::Cancelled),
    }
}

const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "mpeg", "mpg", "ts", "m2ts", "m4v", "3gp",
    "ogv", "mp3", "aac", "flac", "wav", "ogg", "opus", "m4a", "alac", "ac3", "dts", "wma", "ape",
];
