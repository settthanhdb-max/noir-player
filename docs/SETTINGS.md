# Settings Reference

NOIR Player has **176 settings across 10 categories**, all defined as typed
descriptors in `noir-settings` and rendered automatically in the UI. Settings
persist to `settings.toml` (versioned, migrated) in the platform config
directory and are validated on load and on save.

## How the system works

- Each category struct exposes `SettingEntry`s: a dotted key (`video.hwdec`),
  label, description, `SettingKind` (Toggle / Number / Slider / Dropdown /
  Text / Directory / File) and a `SettingValue` borrowing the real field.
- The settings screen is **generated** from these descriptors: search,
  tooltips, per-entry reset, modified-value markers, restart-required markers
  and capability gating all come from the descriptor, not hand-written rows.
- Entries requiring a capability the active engine lacks (e.g. video
  equalizer on the mock engine) render **disabled** with the requirement
  shown — never fake-functional.
- Engine-affecting settings are mapped to `EngineCommand`s live; the rest
  apply in the UI layer immediately. Entries that cannot be applied without a
  restart are marked.

## Categories

### 1. General (17)
Startup screen, restore session, resume playback, confirm exit, always on
top, remember window geometry, start in fullscreen, auto-open last media,
prevent sleep during playback, confirm before clearing history, default media
directory, default screenshot directory, language (English / Tiếng Việt),
date/time format.

### 2. Appearance (23)
Theme (dark / light / follow system), UI scale, font size, compact mode,
dense mode, control bar size, sidebar width, playlist width, corner radius,
animations, show logo, show info overlay, show playback controls, auto-hide
controls + delay, OSD visibility/position/duration.

### 3. Playback (27)
Default/min/max speed, speed increment, default volume, maximum volume policy
(100 / 130 / 200 %), remember volume, auto-resume + resume threshold, seek
intervals (short/long), frame stepping, completion action, auto-play next,
repeat file/playlist, shuffle, pause on minimize / focus loss, resume on
focus, A-B repeat, chapter navigation, bookmarks, history policy, remember
position per file.

### 4. Video (32)
Hardware decoding mode, decoder thread count, video sync, display sync,
frame interpolation, deinterlacing, aspect ratio, crop, zoom, pan, rotation,
flips, brightness/contrast/saturation/gamma/hue, scaling algorithms, dither,
deband, HDR tone mapping, screenshot format and quality. Video-equalizer and
HDR controls are capability-gated.

### 5. Audio (20)
Audio output backend, device, channel layout, audio delay, sync offset,
resampling quality, volume normalization, ReplayGain, amplification,
equalizer (capability-gated), channel downmix, stereo/mono mode, sample rate,
output format, passthrough (capability-gated), preferred audio language,
auto track selection.

### 6. Subtitles (23)
Auto-load external subtitles, preferred languages, track selection, subtitle
delay/sync, font, size, weight, colors, outline, shadow, positioning,
margins, background opacity, ASS override behavior, forced subtitles, hearing
impaired selection, encoding, visibility.

### 7. Network (10)
Timeout, cache size, initial buffering, reconnection behavior, retry count,
streaming buffer, preferred protocols, max concurrent operations. No
credentials are ever stored in the settings file.

### 8. Playlist (13)
Default sorting, auto-add opened files, remove missing files, remember order,
repeat mode, shuffle, show duration/type/file size, double-click behavior,
auto-play selection, save changes, persistent queue.

### 9. Keyboard & Mouse (5 + 28 bindings)
Wheel action (volume / seek / speed), wheel seek step, double-click action,
gestures toggle + sensitivity, plus a **fully editable 28-action keymap** with
capture-mode rebinding, unassign, conflict detection, reset and JSON
profile import/export. Defaults follow the specification (Space, arrows, F, M,
O, P, S, A, `[`, `]`, `0`, Esc).

### 10. Advanced (9)
Advanced mode toggle, custom mpv options (validated before apply), cache
configuration, rendering diagnostics, log verbosity, performance monitoring,
per-category reset, reset all, configuration export/import.

## Persistence & migration

- File: `<config-dir>/noir-player/settings.toml`, `schema_version` field.
- Unknown keys are ignored; wrong-typed values are rejected by validation and
  reset to defaults (never crash, never silently corrupt).
- Version migrations run stepwise; v1 → v2 is implemented and unit-tested.
- Shortcuts profile: `<config-dir>/noir-player/shortcuts.json`.
- Configuration import/export copies the full TOML (Advanced category).
