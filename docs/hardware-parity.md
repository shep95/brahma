# hardware parity

> **last reviewed:** 2026-10-10 · **last updated:** 2026-10-10 · **change:** major (first version: GPU, pen tablets, displays, gestures per platform against photoshop 2026) · **target:** adobe photoshop 2026

each hardware feature photoshop 2026 (27.11.0) uses, per platform, against brahma. photoshop
side: its bundle (metal and opencl paths, "precise color management for HDR display" and EDR
strings, touch bar resources, pen pressure/tilt strings) and adobe's GPU FAQ. brahma side:
`crates/tablet`, `crates/ui-egui/src/stylus.rs`, `apps/brahma/src/gpu_startup.rs`,
`engine/src/display_color.rs`, `apps/brahma/src/monitor_profile.rs`.

**summary (estimated):** ~50% ready. GPU compositing and colour-managed display are solid; pen
input is uneven by platform (windows lacks tilt/rotation, wayland is broken) and there is no HDR
output. remaining: 80–150 h (gap G12). human: tablets (wacom, xp-pen, huion) on each os and an
HDR display to verify.

## GPU

| feature | photoshop | brahma | status |
|---|---|---|---|
| GPU canvas, zoom/pan, rotate view | metal (macos), directx/opencl (windows) | wgpu 30: metal, vulkan, DX12, gl; backend selectable | done |
| GPU compositing of layers, blend modes, effects | partial (photoshop composites mostly on CPU) | wgpu compositor, ≤ 1/255 vs CPU oracle; multichannel and > texture-limit regions fall back to CPU | done (ahead in places) |
| GPU filters (blur gallery, liquify, smart sharpen, camera raw) | yes | CPU (rayon) for filters; GPU for compose and effects | partial |
| documents larger than the GPU texture limit | yes (tiled) | CPU fallback; tiling in progress (#49) | partial |
| CPU-only fallback when no GPU | yes, reduced | `cpu` canvas preference; fails to start without an adapter on some linux systems (#2519, #2412) | partial |
| old / integrated gpus | yes | black line on intel hd (#2426), OpenCore macs fail (#1859), Rgba32Float panic (#2020) | partial |

## pen tablets

| feature | photoshop | macos | windows | linux X11 | linux wayland | web |
|---|---|---|---|---|---|---|
| pressure | yes | yes (AppKit monitor) | yes (WM_POINTER), jumpy (#2046) | yes (xinput2) | via xwayland only; GNOME 50 pen doesn't click (#2622), KDE (#2297) | yes |
| tilt | yes | yes | **no** | yes | no | yes |
| barrel rotation | yes | yes (not applied to tip angle, #1301) | no | partial | no | yes (twist) |
| eraser end | yes | yes | no (#2285) | partial | no | partial |
| WinTab driver mode | yes | – | no | – | – | – |
| pressure curve | windows ink / driver | global curve in preferences (#2195) | same | same | same | same |

## displays and colour

| feature | photoshop | brahma | status |
|---|---|---|---|
| colour-managed canvas (document → monitor profile) | yes | yes, folded into the GPU 3d LUT | done |
| monitor profile detection | per display, live | macos per display at launch; others srgb or a chosen .icc | partial |
| HDR / EDR output for 32-bit and HDR images | yes ("precise color management for HDR display") | no; 8-bit srgb-encoded canvas texture | missing |
| high-DPI, physical 100% | yes | yes (#1943) | done |
| multiple monitors: documents in separate os windows | yes | yes (`extra_windows`) | done |
| panels in separate os windows | yes | in-app floating only (ui-217-5) | missing |

## input devices

| feature | photoshop | brahma | status |
|---|---|---|---|
| trackpad pinch zoom, two-finger pan | yes | yes; pinch scrolls instead in 0.6.0 (#2463) | partial |
| trackpad rotate (rotate view) | yes | yes (`rotate_view.rs`) | done |
| touch bar | yes (intel macs) | no | missing (low value) |
| touch screens (windows) | yes | basic pointer only | partial |
| 3d mouse, MIDI, surface dial | surface dial on windows | no | missing (low value) |
| scanners (WIA / TWAIN) | WIA menu (windows) | menu item live; no driver integration | partial |

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-10 | major | first version |
