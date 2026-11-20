# NOIR Player

**NOIR Player** is a professional, high-performance desktop media player for
Windows 10/11 x64, written in Rust. It pairs the playback power of
[libmpv](https://mpv.io) with a minimalist, fully monochrome interface built on
[egui](https://github.com/emilk/egui) — compact, information-dense, and deeply
customizable.

```
+------------------------------------------------------------------------------+
| NOIR   Big Buck Bunny.mkv            Playing      Home  ...                  |
+------------------------------------------------------------------------------+
|                                                                              |
|                     video viewport (letterboxed)                             |
|                                                                              |
|         control bar auto-hides after 10 s of mouse inactivity                |
+------------------------------------------------------------------------------+
| ------------------*------------------------------    3:24 / 10:56             |
|  |<  [>]  []  >| |  [+]------o------  |  [Playback settings..]  [ii] [Full] |
+------------------------------------------------------------------------------+
```

During playback a single **auto-hiding control bar** sits below the video.
It is visible by default and hides after 4 seconds without input (the delay
is configurable in Settings → Appearance). Moving the mouse brings it
back instantly and the video expands to fill the reclaimed space; hotkeys
keep it alive while it is visible but never summon it mid-film, so seeking
or changing volume by keyboard never interrupts the picture. There are no
taskbars and no on-screen pop-up menus — right-click is intentionally
inert. A **More settings** expander inside the bar opens
visuals/speed/scaling/audio adjustments that apply **live** — playback is
never paused, and no confirmation is needed. A **Restore defaults** button
reverts the visuals to their defaults, and the Settings screen now has
**Apply settings** so configuration changes take effect immediately without
restarting. Keyboard shortcuts keep working. On Windows, update with
`scripts\update-windows.ps1` (pull, build, replace the installed exe) and
see [BUILDING.md](BUILDING.md) for the installer.

## Highlights

- **Real playback engine** — libmpv behind a `PlaybackEngine` trait: hardware
  decoding, subtitles, audio/video track selection, chapters, streaming.
- **Explicit playback state machine** — `Idle → Loading → Playing ⇄ Paused →
  Buffering → Ended / Error`, with the UI reacting to engine events instead of
  polling decoder internals.
- **179 settings across 10 categories**, generated from typed descriptors with
  search, capability gating, reset, import/export, validation and migration.
- **Fully customizable keyboard shortcuts** with conflict detection and
  import/export.
- **Media library** backed by SQLite with background directory scanning,
  history, favorites and resume positions.
- **Playlist manager** with M3U/M3U8 import/export, repeat modes and shuffle.
- **41 UI languages** — English and Vietnamese fully translated, plus 39
  further languages (Chinese, Japanese, Korean, Spanish, Arabic, Hindi,
  Thai…) for the core player chrome, with script-correct fonts bundled.

## Screenshots of the architecture, not the pixels

| Crate           | Responsibility                                                        |
| --------------- | --------------------------------------------------------------------- |
| `noir-core`     | Shared models, errors, playback state machine, event types            |
| `noir-settings` | Typed settings schema, descriptors, persistence, migration, shortcuts |
| `noir-playback` | `PlaybackEngine` trait, controller worker thread, mpv backend, mock   |
| `noir-library`  | SQLite database, directory scanner, M3U, playback history             |
| `noir-platform` | Paths, filesystem helpers, native window embedding                    |
| `noir-ui`       | egui application: theme, i18n, screens, settings UI, input            |

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full design,
[docs/SUPPORTED_FORMATS.md](docs/SUPPORTED_FORMATS.md) for codec support,
[docs/SETTINGS.md](docs/SETTINGS.md) for the settings reference and
[docs/PERFORMANCE.md](docs/PERFORMANCE.md) for performance design notes.

## Building

```sh
cargo build --release
```
During playback a single **auto-hiding control bar** sits below the video:
it is visible by default, hides after 4 seconds without input, and the
video expands to reclaim the space while it is hidden (mouse movement
summons it back; hotkeys do not). No taskbars and no on-screen pop-up
menus — right-click is inert. The bar's
**More settings** expander reveals visuals/speed/scaling/audio adjustments
that apply live, without pausing playback. Keyboard shortcuts keep working.


The mpv backend requires libmpv (`libmpv-dev` / `mpv-devel` / an mpv SDK).
Build without it — using the honest built-in mock engine — with:

```sh
cargo build --release --no-default-features
```

Full instructions, including Windows setup, are in
[BUILDING.md](BUILDING.md).

## Running

```sh
cargo run --release
```

First launch creates `settings.toml` and `library.db` under the platform
config/data directory (`%APPDATA%\noir-player\` on Windows).

## Keyboard shortcuts (defaults)

| Key         | Action                    |
| ----------- | ------------------------- |
| `Space`     | Play / pause              |
| `←` / `→`   | Seek backward / forward   |
| `↑` / `↓`   | Volume up / down          |
| `F`         | Fullscreen                |
| `M`         | Mute                      |
| `O`         | Open file                 |
| `P`         | Playlist panel            |
| `S`         | Subtitle menu             |
| `A`         | Audio track menu          |
| `[` / `]`   | Slower / faster           |
| `0`         | Reset speed               |
| `Esc`       | Exit fullscreen / dialog  |

Every binding is remappable in **Settings → Keyboard & Mouse**.

## Platform support

Windows 10/11 x64 is the primary target. Linux (X11/Wayland) and macOS build
and run with the same code; the mpv child-window embedding path is implemented
for all three, but only Windows is exercised in CI-style testing.

## License

NOIR Player is distributed under the **GPL-2.0-or-later** license — see
[LICENSE](LICENSE). This choice follows from dynamically linking libmpv,
which is commonly built as GPLv2+; details and the third-party inventory are
in [docs/LICENSES.md](docs/LICENSES.md).
