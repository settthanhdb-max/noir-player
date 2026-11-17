# NOIR Player Architecture

## Overview

NOIR Player is a Cargo workspace of six crates with strict, one-directional
dependencies:

```
                ┌───────────┐
                │  noir-ui  │  egui application (binary: noir-player)
                └────┬──────┘
   ┌─────────────┬───┴──────┬──────────────┐
   ▼             ▼          ▼              ▼
┌──────────────┐ ┌──────────────────┐ ┌───────────────┐
│noir-playback │ │  noir-library    │ │ noir-settings │
│ engine trait │ │ SQLite, scanner, │ │ schema,       │
│ mpv / mock   │ │ history, M3U     │ │ descriptors   │
└──────┬───────┘ └────────┬─────────┘ └───────┬───────┘
       ▼                  ▼                   ▼
     ┌───────────────────────────────────────────┐
     │                noir-core                  │
     │  models · errors · PlaybackState · events │
     └───────────────────────────────────────────┘
       ▲
     ┌───────────────┐
     │ noir-platform │  paths · filesystem · native window embedding
     └───────────────┘
```

Dependency rules (enforced by review, kept simple by design):

1. **The UI never touches decoder internals.** `noir-ui` talks to playback
   exclusively through `PlaybackController`, which forwards `EngineCommand`s
   and exposes an immutable `PlaybackSnapshot`.
2. **The playback engine knows nothing about egui.** `noir-playback` depends
   only on `noir-core` (plus libmpv behind the `mpv` feature).
3. **Settings are presentation-independent.** `noir-settings` owns the typed
   schema; the UI renders it from descriptors.
4. **Playback state is explicit.** `noir_core::state::PlaybackState` is the
   single source of truth for `Idle | Loading | Playing | Paused | Buffering |
   Ended | Error`.

## Playback subsystem

### PlaybackEngine trait

```rust
pub trait PlaybackEngine: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> &Capabilities;
    fn subscribe(&self) -> Receiver<EngineEvent>;
    fn send(&self, command: EngineCommand) -> Result<(), PlaybackError>;
    fn snapshot(&self) -> PlaybackSnapshot;
    fn attach_window(&self, parent: *mut c_void) -> Result<(), PlaybackError>;
    fn run(&self);
    fn request_shutdown(&self);
    fn probe(&self, path: &str) -> Result<MediaProbe, PlaybackError>;
    fn event_poll_hint(&self) -> Option<Duration>;
}
```

Two implementations exist:

- **`MpvEngine`** (feature `mpv`) — wraps `libmpv2`. Video is embedded via the
  mpv `wid` property: the engine receives the parent native window handle and
  mpv creates and owns a native child window. The UI keeps that child aligned
  with the egui viewport rectangle every frame
  (`noir_platform::windowing::position_video_child`), using physical pixels
  (`logical × pixels_per_point`). This strategy was chosen over the render API
  because eframe's `Frame` exposes a `RawWindowHandle`, making native child
  windows reliable and GPU-context-ownership-free: mpv owns its rendering
  context and presents directly into the child window.
- **`MockEngine`** — a stateful simulation used when the `mpv` feature is
  disabled. It advances a virtual clock for playing state so the UI timeline,
  OSD and history paths are exercisable, and it reports itself as a mock in
  the UI. It never pretends to decode media.

`create_default_engine()` picks mpv when compiled in and reports its
capabilities honestly (e.g. `video_eq`, `ab_repeat`, `screenshot`); the
settings UI disables controls the active engine does not support instead of
leaving them fake-functional.

### Controller and threading

`PlaybackController::spawn(Arc<dyn PlaybackEngine>)` starts a worker thread
that owns all engine interaction. The UI thread:

- sends `EngineCommand`s (non-blocking channel),
- drains `EngineEvent`s once per frame,
- reads an atomically updated `PlaybackSnapshot` for rendering.

No blocking engine call, file I/O, database query or media probe ever runs on
the UI thread. Long operations (library scans, file dialogs, M3U parsing of
large files) run on worker threads and report back through channels polled in
`NoirApp::background_tick` / `poll_dialogs`.

### State machine

```
Idle ──load──▶ Loading ──media ready──▶ Playing ⇄ Paused
                 │                        │  ▲
                 │ error                  ▼  │ (buffer underrun/recover)
                 ▼                     Buffering
               Error ◀────────────────────┘
Playing/Buffering ──end of file──▶ Ended ──next queue item──▶ Loading
```

Transitions are produced by the engine (`StateChanged` events) and validated
in `noir_core::state`; the UI only renders state and reacts to it.

## Settings subsystem

`noir-settings` stores a strongly typed `Settings` struct with ten category
structs (general, appearance, playback, video, audio, subtitles, network,
playlist, input, advanced). Each category implements a descriptor provider
returning `SettingEntry`s — `(&str key, label, description, SettingKind,
SettingValue)` where `SettingValue` is a mutable borrow into the struct.

This gives the UI three things for free:

- **Generated settings screens** — `noir-ui::settings_ui` renders every entry
  from its descriptor; adding a field to the schema automatically adds the
  control, search hit and reset target.
- **Validation** — `noir-settings::validation` clamps/normalizes values on
  load and before save; invalid custom mpv options are reported, never
  silently applied.
- **Migration** — persisted files carry a `schema_version`; migrations run
  stepwise (v1 → v2 exists and is tested).

Persistence is a versioned TOML file in the platform config directory plus a
separate shortcuts profile (JSON). Settings that change engine behavior are
re-sent to the controller as `EngineCommand`s on change (`settings_map`); the
rest apply immediately in the UI layer (theme, layout, i18n).

## Library subsystem

- **SQLite** (`rusqlite`, bundled) stores media items, directories, history
  and playlists. WAL mode; all access happens off the UI thread.
- **Scanner** — `walkdir` on a worker thread, extension-based media detection,
  hidden-file pruning, progress events (`Found`, `Persisted`, `Finished`,
  `Failed`).
- **History** — `HistoryPolicy` (off / session only / full) decides what gets
  recorded; entries store path, timestamp, position, duration, completion.
- **M3U/M3U8** — import/export with relative-path preservation.

## UI subsystem

- **Theme** — the exact monochrome palette from the design spec is pinned in
  `theme.rs` and unit-tested byte-for-byte. A fingerprint (palette, scale,
  font size, radius, compact/dense) guards against redundant restyling, and
  `pixels_per_point` is always computed from a captured base to avoid
  compounding.
- **i18n** — `Tr` with English tables and Vietnamese overlays; missing keys
  fall back to English, unknown keys render verbatim (never blank).
- **Input** — `ShortcutRegistry` maps `(modifiers, key)` → `ActionBinding`;
  egui key events are translated by `shortcuts::combo_from`. The keyboard
  editor supports capture, unassign, conflict detection and JSON
  import/export.
- **Dialogs** — `rfd` runs on worker threads; results arrive via `std::sync::mpsc`
  and are polled per frame, so no dialog ever blocks the UI loop.

## Error handling

- `noir_core::errors` defines typed errors for playback, settings and library
  domains (`thiserror`), all human-readable.
- Engine errors surface as `EngineEvent::Playback(PlaybackEvent::Error)` and
  are shown as an error toast; they never panic the UI.
- `unsafe` is denied workspace-wide except for the single `unsafe impl Send`
  required to move a raw window handle into the playback thread
  (`noir-platform`), which is documented at the site.
