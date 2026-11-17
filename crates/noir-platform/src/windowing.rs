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
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
        );
        if child.is_null() {
            return false;
        }
        MoveWindow(child, x, y, width, height, true) != 0
    }
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
