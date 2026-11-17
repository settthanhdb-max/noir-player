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

The Rust GNU toolchain (`x86_64-pc-windows-gnu`) links with `-lmpv`, so the
linker needs a libmpv **import library** (`libmpv.dll.a`) — a bare DLL is not
enough. Two reliable setups:

#### Option A — MSYS2 (matches the ucrt64/mingw toolchain)

If you build with the MSYS2 mingw gcc (e.g. `D:/MSYS2/ucrt64`), install the
libmpv development package in the matching environment:

```sh
pacman -S mingw-w64-ucrt-x86_64-mpv     # ucrt64
# or: pacman -S mingw-w64-x86_64-mpv    # msys2/mingw64
```

This provides `libmpv.dll.a` in the environment's `lib` directory (found by
the linker automatically) and `libmpv-2.dll` in its `bin` directory. Then:

```powershell
cargo build --release
```

When running the exe outside the MSYS2 shell, make sure the MSYS2 `bin`
directory (e.g. `D:\MSYS2\ucrt64\bin`) is on `PATH` so `libmpv-2.dll` and its
dependencies resolve.

#### Option B — shinchiro mpv-dev package

1. Download `mpv-dev-x86_64-*` from <https://mpv.io/installation/>.
2. Generate an import library from the shipped `.def` file (requires MSYS2
   binutils, already present with the GNU toolchain):

   ```sh
   dlltool -d libmpv-2.def -D libmpv-2.dll -l libmpv.dll.a
   ```

3. Point the linker at the directory containing `libmpv.dll.a`:

   ```powershell
   $env:RUSTFLAGS = "-L native=C:\path\to\mpv-dev"
   cargo build --release
   ```

4. Copy `libmpv-2.dll` next to `target\release\noir-player.exe` (it also
   needs its own dependencies — the mpv-dev archive lists them; the easiest
   complete set is the full mpv package's `bin` directory on `PATH`).

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
