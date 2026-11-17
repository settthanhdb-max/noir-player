# Performance Design Notes

## Principles applied

1. **Event-driven, not polled.** The UI drains engine events once per frame
   from a channel; the engine worker thread blocks on `mpv.wait_event` with a
   timeout rather than busy-polling. When nothing plays and nothing happens,
   egui does not repaint at all (no `request_repaint` is issued).
2. **Repaint only while it matters.** `NoirApp` requests a 100 ms repaint
   window only while state is `Playing | Buffering | Loading`; on
   `Idle | Paused | Ended | Error` the app is fully idle (0 % CPU from the UI).
3. **Nothing blocking on the UI thread.** Library scans (walkdir), SQLite
   queries, M3U parsing, file dialogs (rfd) and media probing all run on
   worker threads and report through channels polled per frame.
4. **Minimal per-frame allocation.** The theme is only re-applied when a
   fingerprint of appearance settings changes; queue and library lists are
   sorted/filtered once per input change, not per widget.
5. **Video rendering is mpv's job.** The engine owns its rendering context and
   presents into a native child window (`wid` embedding). The UI never copies
   frames; per-frame UI work is a `MoveWindow` alignment call on Windows.

## Memory

- `PlaybackEvent::MediaLoaded` (the largest event) is boxed inside
  `EngineEvent` so event buses stay small.
- Track/chapter lists are fetched as indexed scalar properties instead of
  building full node trees.
- Release profile: `opt-level = 3`, thin LTO, `codegen-units = 1`,
  `panic = "abort"`, debuginfo stripped.

## Safety net

- `overflow-checks` stay on in all profiles (workspace lints deny
  `unsafe_code` everywhere except the single documented `Send` impl for the
  window handle).
- All engine writes go through `EngineCommand`s so the mpv handle is only
  touched on the engine thread (libmpv's API is not generally thread-safe for
  concurrent property access).

## Known costs, stated honestly

- The 100 ms repaint cadence during playback means the on-screen clock can
  lag up to ~100 ms behind real time; this trades negligible idle CPU for
  smooth-enough UI updates. The timeline itself uses engine-issued
  `PositionChanged` events, so seeking accuracy is unaffected.
- `rfd` dialogs are synchronous in nature; NOIR Player runs them on worker
  threads, which adds a tiny latency to dialog results but keeps the UI
  responsive even over slow network shares.
- The mock engine (no-mpv builds) simulates a playback clock; it is a
  development/test aid, not a low-resource player mode.
