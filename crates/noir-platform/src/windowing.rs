//! Windowing helpers (window handle extraction for renderer attachment).
//!
//! eframe/egui expose the native window through raw-window-handle. The
//! extracted handle is passed to the playback engine's `attach_window`.
#![allow(unsafe_code)]

/// Opaque window handle container, platform-neutral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowHandle(pub *mut std::ffi::c_void);

// The handle is only used to configure the renderer; it never dereferences
// the pointee on the sending thread.
unsafe impl Send for WindowHandle {}

impl WindowHandle {
    pub fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

/// Reposition the renderer's native child window (created by the playback
/// backend when embedding via `wid`) so video fills exactly the UI viewport.
///
/// Returns `false` when there is no child window yet or the platform does not
/// support repositioning.
#[cfg(windows)]
pub fn position_video_child(
    parent: *mut std::ffi::c_void,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowExW, MoveWindow};
    unsafe {
        let child = FindWindowExW(
            parent as windows_sys::Win32::Foundation::HWND,
            0,
            std::ptr::null(),
            std::ptr::null(),
        );
        if child == 0 {
            return false;
        }
        MoveWindow(child, x, y, width, height, 1) != 0
    }
}

/// Show or hide the renderer's native child window. The video surface must
/// only be visible while the player screen is active; on any other screen the
/// child would be repositioned over unrelated UI and cover it.
///
/// Returns `true` when a child window exists and the visibility command was
/// delivered, `false` when there is no child window yet.
#[cfg(windows)]
pub fn set_video_child_visible(parent: *mut std::ffi::c_void, visible: bool) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowExW, ShowWindow, SW_HIDE, SW_SHOW,
    };
    unsafe {
        let child = FindWindowExW(
            parent as windows_sys::Win32::Foundation::HWND,
            0,
            std::ptr::null(),
            std::ptr::null(),
        );
        if child == 0 {
            return false;
        }
        ShowWindow(child, if visible { SW_SHOW } else { SW_HIDE });
        true
    }
}

#[cfg(not(windows))]
pub fn set_video_child_visible(_parent: *mut std::ffi::c_void, _visible: bool) -> bool {
    false
}

#[cfg(not(windows))]
pub fn position_video_child(
    _parent: *mut std::ffi::c_void,
    _x: i32,
    _y: i32,
    _width: i32,
    _height: i32,
) -> bool {
    false
}

/// Current mouse cursor position in the client coordinates of `parent`, when
/// the platform can provide it.
///
/// While the embedded native video child is visible, Windows delivers mouse
/// messages to that child window, not to the egui window — so the UI cannot
/// rely on `Context::input` pointer deltas to detect movement over the video.
/// Polling the OS cursor position makes "mouse moved" detection work anywhere
/// over the player window.
///
/// Returns `None` when the platform does not support cursor polling or the
/// call fails.
#[cfg(windows)]
pub fn cursor_client_pos(parent: *mut std::ffi::c_void) -> Option<(i32, i32)> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    unsafe {
        let mut pt = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut pt) == 0 {
            return None;
        }
        ScreenToClient(parent as windows_sys::Win32::Foundation::HWND, &mut pt);
        Some((pt.x, pt.y))
    }
}

#[cfg(not(windows))]
pub fn cursor_client_pos(_parent: *mut std::ffi::c_void) -> Option<(i32, i32)> {
    None
}
