# Third-Party Licenses

NOIR Player is licensed **GPL-2.0-or-later** (see the root `LICENSE`). This
document explains why and inventories the dependencies.

## Why GPL-2.0-or-later

NOIR Player dynamically links **libmpv**, the client library of mpv. The
libmpv *header/API* can be built under LGPL-2.1-or-later, but the libmpv
binaries users actually install (including every mainstream Windows build) are
compiled **with GPL components** (FFmpeg GPL posture, common mpv build
configuration) and are therefore **GPLv2+** as a whole.

A Rust wrapper's license does **not** determine the license of the native
library it binds: `libmpv2` being MIT/Apache-2.0 changes nothing about the
GPL status of the `libmpv-2.dll`/`libmpv.so` that gets loaded at runtime.

Since NOIR Player is distributed as a work that dynamically links that
library, the combined work must comply with GPLv2. We therefore license
NOIR Player as **GPL-2.0-or-later**. If you build and link against an
LGPL-only libmpv, you may relicense your own forks accordingly, but the
upstream default assumes the GPL build.

## Native components

| Component  | License (typical binary) | Role                              |
| ---------- | ------------------------ | --------------------------------- |
| libmpv     | GPLv2+ (LGPLv2.1+ only for special LGPL builds) | Playback engine |
| FFmpeg     | LGPL-2.1+ / GPL (build-dependent) | Demuxing, decoding       |
| libass     | ISC                      | ASS/SSA subtitle rendering        |
| ffmpeg codecs | various               | Codec implementations             |

These are runtime dependencies of the user's installation; NOIR Player does
not redistribute them.

## Rust crates (compile-time dependencies)

| Crate                 | Version | License            |
| --------------------- | ------- | ------------------ |
| eframe / egui         | 0.37    | MIT OR Apache-2.0  |
| glow                  | 0.17    | MIT OR Apache-2.0  |
| libmpv2 / libmpv2-sys | 7.0/4.0 | MIT                |
| serde / serde_json    | 1.0     | MIT OR Apache-2.0  |
| toml                  | 1.2     | MIT OR Apache-2.0  |
| thiserror / anyhow    | 2.0/1.0 | MIT OR Apache-2.0  |
| tracing / tracing-subscriber | 0.3 | MIT OR Apache-2.0 |
| directories           | 6.0     | MIT OR Apache-2.0  |
| rfd                   | 0.15    | MIT                |
| walkdir               | 2.5     | MIT OR Apache-2.0  |
| rusqlite (+ bundled SQLite) | 0.43 | MIT (SQLite: public domain) |
| crossbeam-channel     | 0.5     | MIT OR Apache-2.0  |
| raw-window-handle     | 0.6     | MIT OR Apache-2.0  |
| parking_lot           | 0.12    | MIT OR Apache-2.0  |
| image                 | 0.25    | MIT OR Apache-2.0  |
| windows-sys           | 0.52    | MIT OR Apache-2.0  |

A complete, exact machine-generated inventory (including transitive
dependencies) can be produced with:

```sh
cargo install cargo-about        # or cargo-license
cargo about generate             # run in the workspace root
```

## Trademarks

NOIR Player is an original work. It is **not** affiliated with, and contains
no assets from, PotPlayer, mpv, or any other media player. PotPlayer is used
as a functional design reference only.
