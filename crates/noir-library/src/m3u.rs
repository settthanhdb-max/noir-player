//! M3U / M3U8 playlist import and export.

use std::path::Path;

/// One parsed entry: a path (possibly a URL) and optional display name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M3uEntry {
    pub path: String,
    pub title: Option<String>,
}

/// Parse an M3U/M3U8 document. Handles `#EXTM3U`, `#EXTINF` metadata and
/// plain path lines. Relative paths are resolved against the playlist's
/// parent directory.
pub fn parse(content: &str, base_dir: Option<&Path>) -> Vec<M3uEntry> {
    let mut entries = Vec::new();
    let mut pending_title: Option<String> = None;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("#EXTM3U") {
            continue;
        }
        if let Some(rest) = line.strip_prefix("#EXTINF:") {
            // #EXTINF:<duration>,<title>
            let title = rest.split_once(',').map(|(_, t)| t.trim().to_string());
            pending_title = title.filter(|t| !t.is_empty());
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let path = resolve_path(line, base_dir);
        entries.push(M3uEntry {
            path,
            title: pending_title.take(),
        });
    }
    entries
}

/// Serialize entries to an M3U document with EXTM3U/EXTINF headers.
pub fn serialize(entries: &[M3uEntry], base_dir: Option<&Path>) -> String {
    let mut out = String::from("#EXTM3U\n");
    for entry in entries {
        if let Some(title) = &entry.title {
            out.push_str(&format!("#EXTINF:-1,{title}\n"));
        }
        let path = match base_dir {
            Some(base) if !is_url(&entry.path) => make_relative(Path::new(&entry.path), base),
            _ => entry.path.clone(),
        };
        out.push_str(&path);
        out.push('\n');
    }
    out
}

fn is_url(path: &str) -> bool {
    path.contains("://")
}

fn resolve_path(line: &str, base_dir: Option<&Path>) -> String {
    if is_url(line) || base_dir.is_none() {
        return line.to_string();
    }
    let p = Path::new(line);
    if p.is_absolute() {
        return line.to_string();
    }
    base_dir.unwrap().join(p).to_string_lossy().to_string()
}

fn make_relative(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_and_extinf() {
        let content = "#EXTM3U\n#EXTINF:123,Movie Title\n/movie/a.mkv\n/b.mp3\n";
        let entries = parse(content, None);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title.as_deref(), Some("Movie Title"));
        assert_eq!(entries[0].path, "/movie/a.mkv");
        assert_eq!(entries[1].title, None);
    }

    #[test]
    fn relative_paths_resolve_against_base() {
        let entries = parse("a.mkv\n", Some(Path::new("/media/films")));
        assert_eq!(entries[0].path, "/media/films/a.mkv");
    }

    #[test]
    fn roundtrip() {
        let entries = vec![
            M3uEntry {
                path: "/a.mkv".into(),
                title: Some("A".into()),
            },
            M3uEntry {
                path: "/b.mkv".into(),
                title: None,
            },
        ];
        let text = serialize(&entries, None);
        let back = parse(&text, None);
        assert_eq!(back, entries);
    }

    #[test]
    fn urls_are_kept_absolute() {
        let entries = parse("https://example.com/stream.m3u8\n", Some(Path::new("/x")));
        assert_eq!(entries[0].path, "https://example.com/stream.m3u8");
    }
}
