//! Single-instance coordination.
//!
//! Windows installers associate media types with the player executable, so a
//! double-clicked file would normally start a second process while one is
//! already playing. Instead the launch protocol works like this:
//!
//! 1. The first instance acquires a named mutex and keeps it alive.
//! 2. A second instance fails to acquire it, writes an open request file into
//!    the shared data directory and exits.
//! 3. The running instance polls the request file every frame and opens the
//!    files in its queue.
#![allow(unsafe_code)]

use std::io;
use std::path::Path;

/// Returned when another instance already holds the singleton lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlreadyRunning;

/// Request file holding newline-separated media paths to open.
pub fn open_request_file(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("ipc_open.txt")
}

/// Write an open request so a running instance picks the files up.
pub fn signal_open_request(data_dir: &Path, files: &[String]) -> io::Result<()> {
    let list = files.join("\n");
    let tmp = data_dir.join("ipc_open.txt.new");
    std::fs::write(&tmp, list)?;
    std::fs::rename(&tmp, open_request_file(data_dir))
}

/// Handle guarding the single-instance mutex. Released on drop.
#[cfg(windows)]
pub struct InstanceGuard {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl Drop for InstanceGuard {
    fn drop(&mut self) {
        use windows_sys::Win32::Foundation::CloseHandle;
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

/// Try to become the single running instance. Returns `Err(())` when another
/// instance already holds the lock.
#[cfg(windows)]
pub fn acquire(name: &str) -> Result<InstanceGuard, AlreadyRunning> {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let handle = unsafe { CreateMutexW(std::ptr::null(), 0, wide.as_ptr()) };
    if handle == 0 {
        return Err(AlreadyRunning);
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            CloseHandle(handle);
        }
        return Err(AlreadyRunning);
    }
    Ok(InstanceGuard { handle })
}

#[cfg(not(windows))]
pub struct InstanceGuard;

#[cfg(not(windows))]
pub fn acquire(_name: &str) -> Result<InstanceGuard, AlreadyRunning> {
    Ok(InstanceGuard)
}
