//! Localization.
//!
//! English is the default; Vietnamese is fully supported; 39 further
//! languages ship a core chrome table (see [`crate::locales`]). Strings are
//! looked up by stable keys; keys a language does not translate fall back to
//! English — never to the raw key.
//!
//! The active language is stored as an index into
//! [`noir_settings::schema::general::LANGUAGE_OPTIONS`], which must stay in
//! lockstep with [`crate::locales::LANGUAGES`] (a unit test below enforces
//! that). Index 1 remains Vietnamese so settings written by versions that
//! only shipped English + Vietnamese keep working.

use crate::locales::LANGUAGES;

/// A translated-string handle for the active UI language.
#[derive(Debug, Clone, Copy)]
pub struct Tr {
    /// Index into `LANGUAGE_OPTIONS` / `LANGUAGES` (clamped English).
    pub lang_index: usize,
}

impl Tr {
    /// Build a translator for the language at `index`. Out-of-range indices
    /// (which can only happen with a hand-edited settings file) fall back to
    /// English.
    pub fn from_index(index: usize) -> Self {
        let lang_index = if index < LANGUAGES.len() { index } else { 0 };
        Self { lang_index }
    }

    /// Translate `key`. Unknown keys return the key itself so missing
    /// translations are visible instead of silent.
    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        match self.lang_index {
            0 => en(key),
            1 => vi(key).unwrap_or_else(|| en(key)),
            i => lookup(LANGUAGES[i].table, key).unwrap_or_else(|| en(key)),
        }
    }
}

/// Linear scan over a small static table (≤ 60 entries) — fast enough that
/// a hash map would be overhead.
fn lookup(table: &'static [(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

fn en(key: &str) -> &str {
    match key {
        "app.name" => "NOIR Player",
        "nav.home" => "Home",
        "nav.player" => "Player",
        "nav.library" => "Library",
        "nav.playlists" => "Playlists",
        "nav.history" => "History",
        "nav.settings" => "Settings",
        "state.idle" => "Idle",
        "state.loading" => "Loading",
        "state.playing" => "Playing",
        "state.paused" => "Paused",
        "state.buffering" => "Buffering",
        "state.ended" => "Ended",
        "state.error" => "Error",
        "ctrl.play" => "Play",
        "ctrl.pause" => "Pause",
        "ctrl.stop" => "Stop",
        "queue.remove" => "Remove from queue",
        "ctrl.prev" => "Previous",
        "ctrl.next" => "Next",
        "ctrl.mute" => "Mute",
        "ctrl.unmute" => "Unmute",
        "ctrl.volume" => "Volume",
        "ctrl.speed" => "Speed",
        "ctrl.fullscreen" => "Fullscreen",
        "ctrl.exit_fullscreen" => "Exit fullscreen",
        "ctrl.open_file" => "Open file",
        "ctrl.open_folder" => "Open folder",
        "ctrl.playlist" => "Playlist",
        "ctrl.more" => "More settings",
        "ctrl.show_controls" => "Show controls",
        "menu.playback_settings" => "Playback settings\u{2026}",
        "menu.apply" => "Apply Settings",
        "menu.reset" => "Restore defaults",
        "menu.close" => "Close",
        "menu.scaling" => "Scaling",
        "menu.speed" => "Speed",
        "menu.visuals" => "Visuals",
        "menu.audio" => "Audio",
        "menu.media_info" => "Media info",
        "ctrl.step_back" => "Frame step back",
        "ctrl.step_fwd" => "Frame step forward",
        "ctrl.screenshot" => "Screenshot",
        "ctrl.ab_mark" => "A-B repeat mark",
        "ctrl.info" => "Media information",
        "ctrl.chapter_prev" => "Previous chapter",
        "ctrl.chapter_next" => "Next chapter",
        "time.position" => "Position",
        "time.duration" => "Duration",
        "home.title" => "Home",
        "home.quick_open" => "Quick open",
        "home.continue_watching" => "Continue watching",
        "home.recent" => "Recently played",
        "home.frequent" => "Frequently played",
        "home.favorites" => "Favorites",
        "home.playlists" => "Playlists",
        "home.library" => "Media library",
        "home.empty" => "Nothing played yet. Open a file to get started.",
        "lib.title" => "Media Library",
        "lib.directories" => "Watched directories",
        "lib.add_dir" => "Add directory",
        "lib.remove_dir" => "Remove",
        "lib.rescan" => "Rescan all",
        "lib.scan_status" => "Scan",
        "lib.search" => "Search library",
        "lib.filter_all" => "All",
        "lib.filter_video" => "Video",
        "lib.filter_audio" => "Audio",
        "lib.filter_favorites" => "Favorites",
        "lib.recently_added" => "Recently added",
        "lib.empty" => "No media indexed. Add a directory to scan.",
        "lib.files" => "files",
        "lib.favorite" => "Favorite",
        "lib.unfavorite" => "Unfavorite",
        "lib.remove_missing" => "Remove missing files",
        "pl.title" => "Playlist Manager",
        "pl.queue" => "Queue",
        "pl.saved" => "Saved playlists",
        "pl.clear" => "Clear",
        "pl.save" => "Save queue",
        "pl.load" => "Load",
        "pl.delete" => "Delete",
        "pl.import_m3u" => "Import M3U",
        "pl.export_m3u" => "Export M3U",
        "pl.shuffle" => "Shuffle",
        "pl.repeat_off" => "Repeat off",
        "pl.repeat_one" => "Repeat one",
        "pl.repeat_all" => "Repeat all",
        "pl.empty" => "Queue is empty.",
        "pl.name_prompt" => "Playlist name",
        "pl.move_up" => "Move up",
        "pl.move_down" => "Move down",
        "pl.play_now" => "Play",
        "pl.remove" => "Remove from queue",
        "hist.title" => "Playback History",
        "hist.clear" => "Clear history",
        "hist.empty" => "No playback history.",
        "hist.completion" => "Completion",
        "hist.recording" => "History recording is disabled in settings.",
        "set.title" => "Settings",
        "set.search" => "Search settings",
        "set.reset_category" => "Reset category",
        "set.reset_all" => "Reset all settings",
        "set.apply" => "Apply settings",
        "set.applied" => "Settings applied",
        "set.import" => "Import configuration",
        "set.export" => "Export configuration",
        "set.advanced" => "Show advanced settings",
        "set.restart_required" => "Restart required",
        "set.needs_capability" => "Not supported by the current backend",
        "set.modified" => "Modified",
        "set.default" => "Default",
        "set.reset" => "Reset",
        "set.saved" => "Settings saved",
        "set.error" => "Settings error",
        "set.reset_confirm" => "Reset all settings to defaults?",
        "set.yes" => "Yes",
        "set.no" => "No",
        "set.keys.title" => "Keyboard shortcuts",
        "set.keys.change" => "Change",
        "set.keys.press_key" => "Press a key\u{2026}",
        "set.keys.unassign" => "Unassign",
        "set.keys.reset" => "Reset shortcuts",
        "set.keys.conflict" => "Conflicting shortcuts",
        "set.keys.import" => "Import profile",
        "set.keys.export" => "Export profile",
        "set.capabilities" => "Backend capabilities",
        "dlg.pick_dir" => "Pick directory",
        "dlg.media_files" => "Media files",
        "dlg.all_files" => "All files",
        "dlg.m3u" => "M3U playlists",
        "dlg.json" => "JSON files",
        "misc.confirm_exit" => "Exit NOIR Player?",
        "misc.exit" => "Exit",
        "misc.cancel" => "Cancel",
        "misc.ok" => "OK",
        "misc.drop_hint" => "Drop media files to add them to the queue",
        "misc.no_video" => "No video output available",
        "misc.drop_here" => "Drop files here or press O to open",
        "misc.mock_engine" => "Mock playback engine active \u{2014} no media output. Build with the `mpv` feature and install libmpv for real playback.",
        "misc.now_playing" => "Now playing",
        _ => key,
    }
}

fn vi(key: &str) -> Option<&'static str> {
    Some(match key {
        "app.name" => "NOIR Player",
        "nav.home" => "Trang ch\u{1ee7}",
        "nav.player" => "Tr\u{ec}nh ph\u{e1}t",
        "nav.library" => "Th\u{01b0} vi\u{1ec7}n",
        "nav.playlists" => "Danh s\u{e1}ch ph\u{e1}t",
        "nav.history" => "L\u{1ecb}ch s\u{1eed}",
        "nav.settings" => "C\u{e0}i \u{0111}\u{1eb7}t",
        "state.idle" => "Nh\u{e0}n r\u{1ed7}i",
        "state.loading" => "\u{110}ang t\u{1ea3}i",
        "state.playing" => "\u{110}ang ph\u{e1}t",
        "state.paused" => "T\u{1ea1}m d\u{1eeb}ng",
        "state.buffering" => "\u{110}ang \u{0111}\u{1ec7}m",
        "state.ended" => "\u{110}\u{e3} k\u{1ebf}t th\u{fa}c",
        "state.error" => "L\u{1ed7}i",
        "ctrl.play" => "Ph\u{e1}t",
        "ctrl.pause" => "T\u{1ea1}m d\u{1eeb}ng",
        "ctrl.stop" => "D\u{1eeb}ng",
        "queue.remove" => "X\u{f3}a kh\u{1ecf}i h\u{e0}ng \u{0111}\u{1ee3}i",
        "ctrl.prev" => "Tr\u{01b0}\u{1edb}c",
        "ctrl.next" => "Sau",
        "ctrl.mute" => "T\u{1eaf}t ti\u{1ebf}ng",
        "ctrl.unmute" => "B\u{1ead}t ti\u{1ebf}ng",
        "ctrl.volume" => "\u{c2}m l\u{01b0}\u{1ee3}ng",
        "ctrl.speed" => "T\u{1ed1}c \u{0111}\u{1ed9}",
        "ctrl.fullscreen" => "To\u{e0}n m\u{e0}n h\u{ec}nh",
        "ctrl.exit_fullscreen" => "Tho\u{e1}t to\u{e0}n m\u{e0}n h\u{ec}nh",
        "ctrl.open_file" => "M\u{1edf} t\u{1ec7}p",
        "ctrl.open_folder" => "M\u{1edf} th\u{01b0} m\u{1ee5}c",
        "ctrl.playlist" => "Danh s\u{e1}ch ph\u{e1}t",
        "ctrl.show_controls" => "Hi\u{1ec7}n \u{0111}i\u{1ec1}u khi\u{1ec3}n",
        "ctrl.more" => "Thi\u{1ebf}t l\u{1ead}p kh\u{e1}c",
        "menu.playback_settings" => "C\u{e0}i \u{0111}\u{1eb7}t ph\u{e1}t\u{2026}",
        "menu.apply" => "\u{c1}p d\u{1ee5}ng c\u{e0}i \u{0111}\u{1eb7}t",
        "menu.reset" => "M\u{1eb7}c \u{0111}\u{1ecb}nh",
        "menu.close" => "\u{110}\u{f3}ng",
        "menu.scaling" => "T\u{1ec9} l\u{1ec7} khung",
        "menu.speed" => "T\u{1ed1}c \u{0111}\u{1ed9}",
        "menu.visuals" => "H\u{ec}nh \u{1ea3}nh",
        "menu.audio" => "\u{c2}m thanh",
        "menu.media_info" => "Th\u{f4}ng tin",
        "ctrl.step_back" => "L\u{f9}i m\u{1ed9}t khung h\u{ec}nh",
        "ctrl.step_fwd" => "T\u{1edb}i m\u{1ed9}t khung h\u{ec}nh",
        "ctrl.screenshot" => "Ch\u{1ee5}p m\u{e0}n h\u{ec}nh",
        "ctrl.ab_mark" => "\u{110}\u{e1}nh d\u{1ea5}u l\u{1eb7}p A-B",
        "ctrl.info" => "Th\u{f4}ng tin ph\u{01b0}\u{01a1}ng ti\u{1ec7}n",
        "ctrl.chapter_prev" => "Ch\u{01b0}\u{01a1}ng tr\u{01b0}\u{1edb}c",
        "ctrl.chapter_next" => "Ch\u{01b0}\u{01a1}ng sau",
        "time.position" => "V\u{1ecb} tr\u{ed}",
        "time.duration" => "Th\u{1edd}i l\u{01b0}\u{1ee3}ng",
        "home.title" => "Trang ch\u{1ee7}",
        "home.quick_open" => "M\u{1edf} nhanh",
        "home.continue_watching" => "Xem ti\u{1ebf}p",
        "home.recent" => "V\u{1eeb}a ph\u{e1}t",
        "home.frequent" => "Ph\u{e1}t nhi\u{1ec1}u nh\u{1ea5}t",
        "home.favorites" => "Y\u{ea}u th\u{ed}ch",
        "home.playlists" => "Danh s\u{e1}ch ph\u{e1}t",
        "home.library" => "Th\u{01b0} vi\u{1ec7}n ph\u{01b0}\u{01a1}ng ti\u{1ec7}n",
        "home.empty" => "Ch\u{01b0}a c\u{f3} g\u{ec} \u{0111}\u{01b0}\u{1ee3}c ph\u{e1}t. H\u{e3}y m\u{1edf} m\u{1ed9}t t\u{1ec7}p \u{0111}\u{1ec3} b\u{1eaf}t \u{0111}\u{1ea7}u.",
        "lib.title" => "Th\u{01b0} vi\u{1ec7}n ph\u{01b0}\u{01a1}ng ti\u{1ec7}n",
        "lib.directories" => "Th\u{01b0} m\u{1ee5}c theo d\u{f5}i",
        "lib.add_dir" => "Th\u{ea}m th\u{01b0} m\u{1ee5}c",
        "lib.remove_dir" => "X\u{f3}a",
        "lib.rescan" => "Qu\u{e9}t l\u{1ea1}i t\u{1ea5}t c\u{1ea3}",
        "lib.scan_status" => "Qu\u{e9}t",
        "lib.search" => "T\u{ec}m trong th\u{01b0} vi\u{1ec7}n",
        "lib.filter_all" => "T\u{1ea5}t c\u{1ea3}",
        "lib.filter_video" => "Video",
        "lib.filter_audio" => "\u{c2}m thanh",
        "lib.filter_favorites" => "Y\u{ea}u th\u{ed}ch",
        "lib.recently_added" => "M\u{1edb}i th\u{ea}m",
        "lib.empty" => "Ch\u{01b0}a c\u{f3} ph\u{01b0}\u{01a1}ng ti\u{1ec7}n n\u{e0}o. Th\u{ea}m th\u{01b0} m\u{1ee5}c \u{0111}\u{1ec3} qu\u{e9}t.",
        "lib.files" => "t\u{1ec7}p",
        "lib.favorite" => "Y\u{ea}u th\u{ed}ch",
        "lib.unfavorite" => "B\u{1ecf} y\u{ea}u th\u{ed}ch",
        "lib.remove_missing" => "X\u{f3}a t\u{1ec7}p kh\u{f4}ng t\u{1ed3}n t\u{1ea1}i",
        "pl.title" => "Qu\u{1ea3}n l\u{fd} danh s\u{e1}ch ph\u{e1}t",
        "pl.queue" => "H\u{e0}ng \u{0111}\u{1ee3}i",
        "pl.saved" => "Danh s\u{e1}ch \u{0111}\u{e3} l\u{01b0}u",
        "pl.clear" => "X\u{f3}a h\u{ebf}t",
        "pl.save" => "L\u{01b0}u h\u{e0}ng \u{0111}\u{1ee3}i",
        "pl.load" => "N\u{1ea1}p",
        "pl.delete" => "X\u{f3}a",
        "pl.import_m3u" => "Nh\u{1ead}p M3U",
        "pl.export_m3u" => "Xu\u{1ea5}t M3U",
        "pl.shuffle" => "X\u{e1}o tr\u{1ed9}n",
        "pl.repeat_off" => "Kh\u{f4}ng l\u{1eb7}p",
        "pl.repeat_one" => "L\u{1eb7}p m\u{1ed9}t",
        "pl.repeat_all" => "L\u{1eb7}p t\u{1ea5}t c\u{1ea3}",
        "pl.empty" => "H\u{e0}ng \u{0111}\u{1ee3}i tr\u{1ed1}ng.",
        "pl.name_prompt" => "T\u{ea}n danh s\u{e1}ch",
        "pl.move_up" => "Chuy\u{1ec3}n l\u{ea}n",
        "pl.move_down" => "Chuy\u{1ec3}n xu\u{1ed1}ng",
        "pl.play_now" => "Ph\u{e1}t",
        "pl.remove" => "X\u{f3}a kh\u{1ecf}i h\u{e0}ng \u{0111}\u{1ee3}i",
        "hist.title" => "L\u{1ecb}ch s\u{1eed} ph\u{e1}t",
        "hist.clear" => "X\u{f3}a l\u{1ecb}ch s\u{1eed}",
        "hist.empty" => "Ch\u{01b0}a c\u{f3} l\u{1ecb}ch s\u{1eed} ph\u{e1}t.",
        "hist.completion" => "Ho\u{e0}n th\u{e0}nh",
        "hist.recording" => "Ghi l\u{1ecb}ch s\u{1eed} \u{0111}ang t\u{1eaf}t trong c\u{e0}i \u{0111}\u{1eb7}t.",
        "set.title" => "C\u{e0}i \u{0111}\u{1eb7}t",
        "set.search" => "T\u{ec}m c\u{e0}i \u{0111}\u{1eb7}t",
        "set.reset_category" => "\u{110}\u{1eb7}t l\u{1ea1}i nh\u{f3}m",
        "set.reset_all" => "\u{110}\u{1eb7}t l\u{1ea1}i t\u{1ea5}t c\u{1ea3}",
        "set.apply" => "\u{c1}p d\u{1ee5}ng c\u{e0}i \u{0111}\u{1eb7}t",
        "set.applied" => "\u{110}\u{e3} \u{e1}p d\u{1ee5}ng c\u{e0}i \u{0111}\u{1eb7}t",
        "set.import" => "Nh\u{1ead}p c\u{1ea5}u h\u{ec}nh",
        "set.export" => "Xu\u{1ea5}t c\u{1ea5}u h\u{ec}nh",
        "set.advanced" => "Hi\u{1ec7}n c\u{e0}i \u{0111}\u{1eb7}t n\u{e2}ng cao",
        "set.restart_required" => "C\u{1ea7}n kh\u{1edf}i \u{0111}\u{1ed9}ng l\u{1ea1}i",
        "set.needs_capability" => "Backend hi\u{1ec7}n t\u{1ea1}i kh\u{f4}ng h\u{1ed7} tr\u{1ee3}",
        "set.modified" => "\u{110}\u{e3} thay \u{0111}\u{1ed5}i",
        "set.default" => "M\u{1eb7}c \u{0111}\u{1ecb}nh",
        "set.reset" => "\u{110}\u{1eb7}t l\u{1ea1}i",
        "set.saved" => "\u{110}\u{e3} l\u{01b0}u c\u{e0}i \u{0111}\u{1eb7}t",
        "set.error" => "L\u{1ed7}i c\u{e0}i \u{0111}\u{1eb7}t",
        "set.reset_confirm" => "\u{110}\u{1eb7}t l\u{1ea1}i t\u{1ea5}t c\u{1ea3} c\u{e0}i \u{0111}\u{1eb7}t v\u{1ec1} m\u{1eb7}c \u{0111}\u{1ecb}nh?",
        "set.yes" => "C\u{f3}",
        "set.no" => "Kh\u{f4}ng",
        "set.keys.title" => "Ph\u{ed}m t\u{1eaf}t",
        "set.keys.change" => "\u{110}\u{1ed5}i",
        "set.keys.press_key" => "Nh\u{1ea5}n m\u{1ed9}t ph\u{ed}m\u{2026}",
        "set.keys.unassign" => "B\u{1ecf} g\u{e1}n",
        "set.keys.reset" => "\u{110}\u{1eb7}t l\u{1ea1}i ph\u{ed}m t\u{1eaf}t",
        "set.keys.conflict" => "Ph\u{ed}m t\u{1eaf}t tr\u{f9}ng nhau",
        "set.keys.import" => "Nh\u{1ead}p h\u{1ed3} s\u{01a1}",
        "set.keys.export" => "Xu\u{1ea5}t h\u{1ed3} s\u{01a1}",
        "set.capabilities" => "Kh\u{1ea3} n\u{0103}ng backend",
        "dlg.pick_dir" => "Ch\u{1ecd}n th\u{01b0} m\u{1ee5}c",
        "dlg.media_files" => "T\u{1ec7}p ph\u{01b0}\u{01a1}ng ti\u{1ec7}n",
        "dlg.all_files" => "T\u{1ea5}t c\u{1ea3} t\u{1ec7}p",
        "dlg.m3u" => "Danh s\u{e1}ch M3U",
        "dlg.json" => "T\u{1ec7}p JSON",
        "misc.confirm_exit" => "Tho\u{e1}t NOIR Player?",
        "misc.exit" => "Tho\u{e1}t",
        "misc.cancel" => "H\u{1ee7}y",
        "misc.ok" => "OK",
        "misc.drop_hint" => "Th\u{1ea3} t\u{1ec7}p ph\u{01b0}\u{01a1}ng ti\u{1ec7}n \u{0111}\u{1ec3} th\u{ea}m v\u{e0}o h\u{e0}ng \u{0111}\u{1ee3}i",
        "misc.no_video" => "Kh\u{f4}ng c\u{f3} \u{0111}\u{1ea7}u ra video",
        "misc.drop_here" => "Th\u{1ea3} t\u{1ec7}p v\u{e0}o \u{0111}\u{e2}y ho\u{1eb7}c nh\u{1ea5}n O \u{0111}\u{1ec3} m\u{1edf}",
        "misc.mock_engine" => "\u{110}ang d\u{f9}ng engine m\u{f4} ph\u{1ecf}ng \u{2014} kh\u{f4}ng c\u{f3} \u{0111}\u{1ea7}u ra ph\u{01b0}\u{01a1}ng ti\u{1ec7}n. H\u{e3}y bi\u{ea}n d\u{1ecb}ch v\u{1edb}i t\u{ed}nh n\u{0103}ng `mpv` v\u{e0} c\u{e0}i libmpv \u{0111}\u{1ec3} ph\u{e1}t th\u{1ead}t.",
        "misc.now_playing" => "\u{110}ang ph\u{e1}t",
        _ => return None,
    })
}

/// Bundle the Noto fallback fonts for scripts the default egui font does not
/// cover (CJK, Arabic, Hebrew, Devanagari, Bengali, Thai, Tamil). They are
/// appended to the fallback chain so Latin rendering (and current metrics)
/// stays unchanged while every other bundled language renders correctly.
pub fn install_fonts(ctx: &egui::Context) {
    let mut defs = egui::FontDefinitions::default();

    const FONTS: &[(&str, &[u8])] = &[
        (
            "noto-sc",
            include_bytes!("../assets/fonts/NotoSansSC-Regular.otf"),
        ),
        (
            "noto-jp",
            include_bytes!("../assets/fonts/NotoSansJP-Regular.otf"),
        ),
        (
            "noto-kr",
            include_bytes!("../assets/fonts/NotoSansKR-Regular.otf"),
        ),
        (
            "noto-arabic",
            include_bytes!("../assets/fonts/NotoSansArabic-Regular.ttf"),
        ),
        (
            "noto-hebrew",
            include_bytes!("../assets/fonts/NotoSansHebrew-Regular.ttf"),
        ),
        (
            "noto-devanagari",
            include_bytes!("../assets/fonts/NotoSansDevanagari-Regular.ttf"),
        ),
        (
            "noto-bengali",
            include_bytes!("../assets/fonts/NotoSansBengali-Regular.ttf"),
        ),
        (
            "noto-thai",
            include_bytes!("../assets/fonts/NotoSansThai-Regular.ttf"),
        ),
        (
            "noto-tamil",
            include_bytes!("../assets/fonts/NotoSansTamil-Regular.ttf"),
        ),
    ];

    for (name, data) in FONTS {
        defs.font_data
            .insert((*name).to_owned(), egui::FontData::from_static(data).into());
    }
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        if let Some(list) = defs.families.get_mut(&family) {
            list.extend(FONTS.iter().map(|(name, _)| (*name).to_owned()));
        }
    }

    ctx.set_fonts(defs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use noir_settings::schema::general::LANGUAGE_OPTIONS;

    #[test]
    fn english_lookup() {
        let tr = Tr::from_index(0);
        assert_eq!(tr.t("app.name"), "NOIR Player");
        assert_eq!(tr.t("nav.player"), "Player");
        assert_eq!(tr.t("menu.reset"), "Restore defaults");
        assert_eq!(tr.t("set.apply"), "Apply settings");
        assert_eq!(tr.t("set.applied"), "Settings applied");
    }

    #[test]
    fn vietnamese_lookup() {
        let tr = Tr::from_index(1);
        assert_eq!(tr.t("nav.home"), "Trang ch\u{1ee7}");
        assert_eq!(
            tr.t("set.apply"),
            "\u{c1}p d\u{1ee5}ng c\u{e0}i \u{0111}\u{1eb7}t"
        );
    }

    #[test]
    fn vietnamese_falls_back_to_english() {
        let tr = Tr::from_index(1);
        // `vi` has no table entry for this key; English applies.
        assert_eq!(tr.t("app.name"), "NOIR Player");
    }

    #[test]
    fn other_languages_translate_core_keys_and_fall_back() {
        // zh-CN is index 2: core keys are translated; keys outside a
        // language's table fall back to English.
        let tr = Tr::from_index(2);
        assert!(!tr.t("nav.home").is_ascii());
        assert_eq!(tr.t("set.reset_confirm"), en("set.reset_confirm"));
        assert_eq!(tr.t("no.such.key"), "no.such.key");
    }

    #[test]
    fn unknown_key_is_returned_verbatim() {
        let tr = Tr::from_index(0);
        assert_eq!(tr.t("no.such.key"), "no.such.key");
    }

    #[test]
    fn out_of_range_index_clamps_to_english() {
        assert_eq!(Tr::from_index(999).lang_index, 0);
        assert_eq!(Tr::from_index(41).lang_index, 0);
        assert_eq!(Tr::from_index(40).lang_index, 40);
    }

    #[test]
    fn language_options_align_with_registry() {
        assert_eq!(LANGUAGE_OPTIONS.len(), LANGUAGES.len());
        for (i, (option, lang)) in LANGUAGE_OPTIONS.iter().zip(LANGUAGES.iter()).enumerate() {
            assert_eq!(
                *option, lang.name,
                "index {i}: option/registry name mismatch"
            );
        }
    }
}
