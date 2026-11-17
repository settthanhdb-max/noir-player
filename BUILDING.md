# Building NOIR Player

## Prerequisites

- **Rust stable** (1.85 or newer is recommended; the workspace pins `stable`
  via `rust-toolchain.toml`, so any recent `rustup` setup works).
- **libmpv** — only when building with the default `mpv` feature.

## Quick start

```sh
git clone <repository-url>
cd noir-player
cargo build --release
```

The resulting binary is `target/release/noir-player(.exe)`.

## Feature flags

| Feature | Default | Effect                                                              |
| ------- | ------- | ------------------------------------------------------------------- |
| `mpv`   | yes     | Enables the real libmpv playback backend (`noir-playback/mpv`).     |

Building **without** the mpv feature is fully supported:

```sh
cargo build --release --no-default-features
```

The application then runs with the built-in `MockEngine`, which implements the
same `PlaybackEngine` trait but does not decode media. It reports its
limitations honestly in the UI (the player screen shows a mock-engine notice
and playback commands are simulated); it exists so the UI, settings and
library subsystems can be developed and tested without native libmpv.

## Installing libmpv

NOIR Player dynamically links `libmpv` (via the `libmpv2` crate) and embeds
its video output as a native child window.

### Windows 10/11

1. Download an mpv dev package, e.g. the `mpv-dev-x86_64-*` archive from
   <https://mpv.io/installation/> (shinchiro builds) or via `winget`/`scoop`.
2. Copy `libmpv-2.dll` next to `noir-player.exe`, or place its directory on
   `PATH`. The import library is not needed at link time because `libmpv2-sys`
   resolves symbols from the DLL at runtime on Windows; on MSVC toolchains a
   `libmpv-2.dll.lib` import library may be required — point `RUSTFLAGS`
   `search` paths at it if needed.
3. `cargo build --release` (default features).

### Linux

```sh
# Debian/Ubuntu
sudo apt install libmpv-dev
# Fedora
sudo dnf install mpv-devel
# Arch
sudo pacman -S mpv
```

### macOS

```sh
brew install mpv
```

## Verification performed in this repository

The development sandbox has no display server and no ability to install
system libmpv, so the following checks are what every change passes:

```sh
cargo check --workspace --no-default-features   # zero errors, zero warnings
cargo clippy --workspace --no-default-features  # zero warnings
cargo test --workspace --no-default-features    # 79 tests
cargo check -p noir-playback --features mpv     # mpv backend type-checks
cargo check -p noir-ui --features mpv           # full app type-checks with mpv
cargo build -p noir-ui --no-default-features    # GUI binary links (mock engine)
```

The `mpv` feature **type-checks but cannot link** on a machine without the
libmpv shared library; that is expected and is why the sandbox verification
uses `--no-default-features` for the link/test steps.

## Cross-compiling Windows from Linux (optional)

```sh
rustup target add x86_64-pc-windows-gnu
# with mingw-w64 installed:
cargo build --release --target x86_64-pc-windows-gnu
```

The mpv feature additionally needs a Windows build of `libmpv-2.dll` and a
matching import library for the GNU linker.

## Release profile

The workspace release profile is already tuned:

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = "debuginfo"
panic = "abort"
overflow-checks = true   # inherited from dev through workspace lints
```

## Running the app

```sh
cargo run --release
```

Logs go to stderr and to a log file in the platform data directory. Control
verbosity with `RUST_LOG` (e.g. `RUST_LOG=debug cargo run`).
