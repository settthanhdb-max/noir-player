//! Localization. English is the default; Vietnamese is fully supported.
//!
//! Strings are looked up by stable keys; missing keys fall back to English.

use noir_settings::schema::general::LANGUAGE_OPTIONS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Vietnamese,
}

impl Language {
    pub fn from_index(index: usize) -> Self {
        match LANGUAGE_OPTIONS.get(index) {
            Some(&"Tiếng Việt") => Self::Vietnamese,
            _ => Self::English,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::English => 0,
            Self::Vietnamese => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tr {
    pub lang: Language,
}

impl Tr {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    /// Translate `key`. Unknown keys return the key itself so missing
    /// translations are visible instead of silent.
    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        match (self.lang, key) {
            (_, "app.name") => "NOIR Player",
            (Language::English, k) => en(k),
            (Language::Vietnamese, k) => vi(k).unwrap_or_else(|| en(k)),
        }
    }
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
        "menu.playback_settings" => "Playback settings…",
        "menu.apply" => "Apply Settings",
        "menu.reset" => "Reset",
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
        "set.keys.press_key" => "Press a key…",
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
        "misc.mock_engine" => "Mock playback engine active — no media output. Build with the `mpv` feature and install libmpv for real playback.",
        "misc.now_playing" => "Now playing",
        _ => key,
    }
}

fn vi(key: &str) -> Option<&'static str> {
    Some(match key {
        "app.name" => "NOIR Player",
        "nav.home" => "Trang chủ",
        "nav.player" => "Trình phát",
        "nav.library" => "Thư viện",
        "nav.playlists" => "Danh sách phát",
        "nav.history" => "Lịch sử",
        "nav.settings" => "Cài đặt",
        "state.idle" => "Nhàn rỗi",
        "state.loading" => "Đang tải",
        "state.playing" => "Đang phát",
        "state.paused" => "Tạm dừng",
        "state.buffering" => "Đang đệm",
        "state.ended" => "Đã kết thúc",
        "state.error" => "Lỗi",
        "ctrl.play" => "Phát",
        "ctrl.pause" => "Tạm dừng",
        "ctrl.stop" => "Dừng",
        "queue.remove" => "Xóa khỏi hàng đợi",
        "ctrl.prev" => "Trước",
        "ctrl.next" => "Sau",
        "ctrl.mute" => "Tắt tiếng",
        "ctrl.unmute" => "Bật tiếng",
        "ctrl.volume" => "Âm lượng",
        "ctrl.speed" => "Tốc độ",
        "ctrl.fullscreen" => "Toàn màn hình",
        "ctrl.exit_fullscreen" => "Thoát toàn màn hình",
        "ctrl.open_file" => "Mở tệp",
        "ctrl.open_folder" => "Mở thư mục",
        "ctrl.playlist" => "Danh sách phát",
        "menu.playback_settings" => "Cài đặt phát…",
        "menu.apply" => "Áp dụng cài đặt",
        "menu.reset" => "Mặc định",
        "menu.close" => "Đóng",
        "menu.scaling" => "Tỉ lệ khung",
        "menu.speed" => "Tốc độ",
        "menu.visuals" => "Hình ảnh",
        "menu.audio" => "Âm thanh",
        "menu.media_info" => "Thông tin",
        "ctrl.step_back" => "Lùi một khung hình",
        "ctrl.step_fwd" => "Tới một khung hình",
        "ctrl.screenshot" => "Chụp màn hình",
        "ctrl.ab_mark" => "Đánh dấu lặp A-B",
        "ctrl.info" => "Thông tin phương tiện",
        "ctrl.chapter_prev" => "Chương trước",
        "ctrl.chapter_next" => "Chương sau",
        "time.position" => "Vị trí",
        "time.duration" => "Thời lượng",
        "home.title" => "Trang chủ",
        "home.quick_open" => "Mở nhanh",
        "home.continue_watching" => "Xem tiếp",
        "home.recent" => "Vừa phát",
        "home.frequent" => "Phát nhiều nhất",
        "home.favorites" => "Yêu thích",
        "home.playlists" => "Danh sách phát",
        "home.library" => "Thư viện phương tiện",
        "home.empty" => "Chưa có gì được phát. Hãy mở một tệp để bắt đầu.",
        "lib.title" => "Thư viện phương tiện",
        "lib.directories" => "Thư mục theo dõi",
        "lib.add_dir" => "Thêm thư mục",
        "lib.remove_dir" => "Xóa",
        "lib.rescan" => "Quét lại tất cả",
        "lib.scan_status" => "Quét",
        "lib.search" => "Tìm trong thư viện",
        "lib.filter_all" => "Tất cả",
        "lib.filter_video" => "Video",
        "lib.filter_audio" => "Âm thanh",
        "lib.filter_favorites" => "Yêu thích",
        "lib.recently_added" => "Mới thêm",
        "lib.empty" => "Chưa có phương tiện nào. Thêm thư mục để quét.",
        "lib.files" => "tệp",
        "lib.favorite" => "Yêu thích",
        "lib.unfavorite" => "Bỏ yêu thích",
        "lib.remove_missing" => "Xóa tệp không tồn tại",
        "pl.title" => "Quản lý danh sách phát",
        "pl.queue" => "Hàng đợi",
        "pl.saved" => "Danh sách đã lưu",
        "pl.clear" => "Xóa hết",
        "pl.save" => "Lưu hàng đợi",
        "pl.load" => "Nạp",
        "pl.delete" => "Xóa",
        "pl.import_m3u" => "Nhập M3U",
        "pl.export_m3u" => "Xuất M3U",
        "pl.shuffle" => "Xáo trộn",
        "pl.repeat_off" => "Không lặp",
        "pl.repeat_one" => "Lặp một",
        "pl.repeat_all" => "Lặp tất cả",
        "pl.empty" => "Hàng đợi trống.",
        "pl.name_prompt" => "Tên danh sách",
        "pl.move_up" => "Chuyển lên",
        "pl.move_down" => "Chuyển xuống",
        "pl.play_now" => "Phát",
        "pl.remove" => "Xóa khỏi hàng đợi",
        "hist.title" => "Lịch sử phát",
        "hist.clear" => "Xóa lịch sử",
        "hist.empty" => "Chưa có lịch sử phát.",
        "hist.completion" => "Hoàn thành",
        "hist.recording" => "Ghi lịch sử đang tắt trong cài đặt.",
        "set.title" => "Cài đặt",
        "set.search" => "Tìm cài đặt",
        "set.reset_category" => "Đặt lại nhóm",
        "set.reset_all" => "Đặt lại tất cả",
        "set.import" => "Nhập cấu hình",
        "set.export" => "Xuất cấu hình",
        "set.advanced" => "Hiện cài đặt nâng cao",
        "set.restart_required" => "Cần khởi động lại",
        "set.needs_capability" => "Backend hiện tại không hỗ trợ",
        "set.modified" => "Đã thay đổi",
        "set.default" => "Mặc định",
        "set.reset" => "Đặt lại",
        "set.saved" => "Đã lưu cài đặt",
        "set.error" => "Lỗi cài đặt",
        "set.reset_confirm" => "Đặt lại tất cả cài đặt về mặc định?",
        "set.yes" => "Có",
        "set.no" => "Không",
        "set.keys.title" => "Phím tắt",
        "set.keys.change" => "Đổi",
        "set.keys.press_key" => "Nhấn một phím…",
        "set.keys.unassign" => "Bỏ gán",
        "set.keys.reset" => "Đặt lại phím tắt",
        "set.keys.conflict" => "Phím tắt trùng nhau",
        "set.keys.import" => "Nhập hồ sơ",
        "set.keys.export" => "Xuất hồ sơ",
        "set.capabilities" => "Khả năng backend",
        "dlg.pick_dir" => "Chọn thư mục",
        "dlg.media_files" => "Tệp phương tiện",
        "dlg.all_files" => "Tất cả tệp",
        "dlg.m3u" => "Danh sách M3U",
        "dlg.json" => "Tệp JSON",
        "misc.confirm_exit" => "Thoát NOIR Player?",
        "misc.exit" => "Thoát",
        "misc.cancel" => "Hủy",
        "misc.ok" => "OK",
        "misc.drop_hint" => "Thả tệp phương tiện để thêm vào hàng đợi",
        "misc.no_video" => "Không có đầu ra video",
        "misc.drop_here" => "Thả tệp vào đây hoặc nhấn O để mở",
        "misc.mock_engine" => "Đang dùng engine mô phỏng — không có đầu ra phương tiện. Hãy biên dịch với tính năng `mpv` và cài libmpv để phát thật.",
        "misc.now_playing" => "Đang phát",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_lookup() {
        let tr = Tr {
            lang: Language::English,
        };
        assert_eq!(tr.t("app.name"), "NOIR Player");
        assert_eq!(tr.t("nav.player"), "Player");
    }

    #[test]
    fn vietnamese_lookup() {
        let tr = Tr {
            lang: Language::Vietnamese,
        };
        assert_eq!(tr.t("nav.home"), "Trang chủ");
    }

    #[test]
    fn vietnamese_falls_back_to_english() {
        let tr = Tr {
            lang: Language::Vietnamese,
        };
        assert_eq!(tr.t("app.name"), "NOIR Player");
    }

    #[test]
    fn unknown_key_is_returned_verbatim() {
        let tr = Tr {
            lang: Language::English,
        };
        assert_eq!(tr.t("no.such.key"), "no.such.key");
    }

    #[test]
    fn language_from_index() {
        assert_eq!(Language::from_index(0), Language::English);
        assert_eq!(Language::from_index(1), Language::Vietnamese);
        assert_eq!(Language::from_index(99), Language::English);
    }
}
