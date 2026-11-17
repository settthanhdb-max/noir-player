# Supported Formats

NOIR Player delegates demuxing, decoding and rendering to **libmpv**, which in
turn uses FFmpeg. Format support is therefore determined by the libmpv build
installed on the user's machine — **not** by NOIR Player itself. This document
states what is confirmed, what is expected, and what is explicitly *not*
claimed.

## The honest rule

> A file extension being recognized by the UI (e.g. in the media library or
> file dialog) is **not** a support claim. Playback support is confirmed only
> by the engine successfully opening the file. When libmpv cannot play
> something, NOIR Player shows the engine's error message in an error toast.

## Confirmed (by the integration contract)

| Aspect        | Status                                                                     |
| ------------- | -------------------------------------------------------------------------- |
| Demux/decode  | Whatever the linked libmpv/FFmpeg supports                                 |
| Video output  | Native child-window embedding (`wid`), mpv-owned rendering context         |
| Audio output  | mpv's audio output layer (`ao`) on the host platform                       |
| Subtitles     | Rendered by libmpv (libass); external files loadable at runtime            |
| Hardware dec. | `hwdec` property passed through (`auto`, `no`, plus backend-specific modes)|

## Expected video containers

MP4, MKV, WebM, AVI, MOV, FLV, WMV, MPEG-PS/TS (MPG, TS, M2TS) — all
demuxed by FFmpeg in every mainstream libmpv build.

## Expected video codecs

H.264/AVC, H.265/HEVC, AV1, VP8, VP9, MPEG-1/2, MPEG-4 Part 2. Hardware
decode path depends on the platform API (D3D11VA/DXVA2 on Windows, VAAPI/VDPAU
on Linux, VideoToolbox on macOS) and the selected `hwdec` mode.

## Expected audio formats

MP3, AAC, FLAC, WAV, OGG/Vorbis, Opus, ALAC, AC-3, DTS — decode and
passthrough availability varies with the audio output backend and device.

## Expected subtitle formats

SRT, ASS/SSA (styled via libass), WebVTT, PGS (bitmap), VobSub/IDX.
ASS styling overrides are configurable; whether they apply is decided by the
`ass-vsfilter`-style options of libmpv, not by NOIR Player.

## Streaming

HTTP(S), HLS, DASH and other protocols supported by the linked FFmpeg build
work through mpv's network layer (`ytdl` hooks are not integrated). Network
cache, timeout and retry settings in NOIR Player map directly onto mpv
properties.

## Platform-specific limitations

- **Windows**: the primary target. D3D11VA hardware decoding is the expected
  default with `hwdec=auto`.
- **Linux**: VAAPI/VDPAU depending on drivers; the child-window embedding uses
  X11 window IDs (Wayland builds of mpv may refuse `wid` embedding — in that
  case mpv opens its own window; this is an mpv/ Wayland limitation, reported
  honestly rather than worked around silently).
- **Sandbox verification**: development CI cannot decode media at all (no
  display, no libmpv); format support above is *contractual* (mpv property
  mapping), not empirically verified per codec in this repository.

## Adding capability awareness

The engine reports `Capabilities` (video equalizer, A-B repeat, screenshots,
audio DSP, HDR passthrough, …). Settings that require a capability the active
engine lacks are shown disabled with their requirement noted — the settings
system never presents an unsupported control as functional.
