//! Filesystem helpers: safe listing, size formatting.

use std::fs;
use std::path::Path;

/// Metadata for one directory entry, read without following symlinks.
#[derive(Debug, Clone, PartialEq)]
pub struct EntryInfo {
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

/// List a directory, returning an error (never a fake empty success) when
/// the directory cannot be read.
pub fn list_dir(dir: &Path) -> std::io::Result<Vec<EntryInfo>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        out.push(EntryInfo {
            path: entry.path(),
            is_dir: meta.is_dir(),
            size: meta.len(),
        });
    }
    out.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
    Ok(out)
}

/// Human-readable file size.
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if bytes == 0 {
        return "0 B".into();
    }
    let exp = (bytes as f64).log(1024.0).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);
    let value = bytes as f64 / 1024f64.powi(exp as i32);
    if exp == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[exp])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_formatting() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(2048), "2.0 KiB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MiB");
    }

    #[test]
    fn list_missing_dir_errors() {
        assert!(list_dir(Path::new("/nonexistent/noir/xyz")).is_err());
    }
}
