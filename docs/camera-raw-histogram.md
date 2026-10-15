# camera raw histogram and scopes

a clean-room implementation of camera raw's SDR histogram interaction and vectorscope,
based on adobe's public documentation. it does not claim numerical identity with adobe's
develop pipeline. automation fields are in [control-protocol.md](control-protocol.md).

| behaviour | implementation |
|---|---|
| RGB histogram | continuous ribbons with additive overlap colours; display-only five-bin binomial smoothing over exact raw counts; edge spikes pinned and excluded from the auto-scale |
| updates, before/after | counted from the same completed float proxy as the preview texture |
| five tonal zones | hover tooltip, horizontal drag and double-click reset of the existing blacks / shadows / exposure / highlights / whites parameters |
| clipping warnings | exact per-channel endpoints (≤0 / ≥1); coloured triangles, u / o, blue / red preview overlays |
| alt/option diagnostics | per-channel shadow or highlight clipping while alt-dragging a light tone slider or a histogram zone |
| pixel hover | tone band and RGB badge on the histogram; ICC-managed crosshair on the vectorscope |
| readouts | RGB or lab (document ICC profile → D50 lab); up to nine movable probes (s, alt-click removes) |
| vectorscope | hidden on opening, enabled from the context menu; 128×64 hue/saturation bins of the whole image or the document selection; red at 3 o'clock and skin-tone line options |
| floating scopes | resizable panel; drag it back onto the dock to restore |
| preferences | display options and panel geometry persist in `dialogs["filter.cameraRaw.scope"]`; probes and vectorscope visibility do not |

the settings panel is grouped as light, color, effects, curve, color mixer, color grading and
detail, with the existing filter parameters and processing. the point curve shares its editing
model with image › adjustments › curves and layer properties (`point_curve.rs`): button-down hit
testing, grab offsets, ordered points, endpoint protection, drag-off removal and re-entry,
keyboard nudges and deletion, at most 16 points. camera raw commits one `filter.cameraRaw` edit on
ok; scope interaction never edits the document. double-clicking a camera raw smart filter reopens
this dialog with its stored settings and the pixels below it; ok updates that filter in place.

## PSD smart filters

camera raw is read and written as a live PSD smart filter (`Adobe Camera Raw Filter`, filter id
2783). the verified subset covers filter-relative temperature/tint with custom white balance,
light, texture/clarity/dehaze/vibrance/saturation, the parametric curve, four point curves,
all eight HSL bands, all four color grading wheels and blending/balance, sharpening,
noise amounts/detail, grain and the numeric post-crop vignette controls. point curves contain
2–16 points with integer input/output levels; resetting one writes the linear endpoints.
PSD stores slider levels and curve coordinates as integers while brahma's sliders are
continuous: PSD export rounds them to the nearest level (hues of 360° are written as 0°),
so a re-opened PSD shows whole numbers. exposure and sharpening radius stay fractional.
the mapping comes from two revisions of a user-supplied photoshop PSD written by camera raw
18.4 (process 6). process 5's
corresponding version pair is also recognized; other versions remain opaque.

imported filters keep their original descriptor in `params.__cameraRawPsd`. unchanged settings,
including the exact stored exposure double, are preserved; editing patches only changed mapped
fields. the template survives `.pcraft`, undo and reopening the camera raw dialog. descriptors
with unrecognized processing fields remain `psd.unsupportedFilter`: their data survives a PSD
save, but their settings cannot be edited or re-rendered by brahma. non-default brahma
settings without a verified PSD mapping cause the existing export warning and omission of that
filter; they are never silently written as default camera raw settings.

active manual lens distortion/vignetting, luminance-noise contrast, color-noise smoothness,
post-crop vignette style (`PCVS`, still unverified) and highlights, RAW kelvin white balance,
masks and HDR still need mapping and/or rendering support. observed neutral-only controls are
retained in the template; non-neutral values keep the filter opaque. in the updated fixture,
distortion −28, lens vignetting 28 and `PCVS=1` keep the entire filter opaque, while all settings
survive PSD and `.pcraft` saves. its supported controls are also checked as a descriptor-only
projection with those three fields removed; that projection does not replace the source filter.

matching settings does not
claim matching pixels: brahma uses its own develop pipeline. PSD round trips and descriptor
patching are checked locally; reopening brahma-generated exports in photoshop is a separate
acceptance check.

## layering and bounds

analysis lives in l2 `brahma-algo` (`histogram`, `vectorscope`), colour transforms in l0
`brahma-cms`, view state in `state.rs`, interaction and painting in `camera_raw_scope_ui.rs`
and `rgb_histogram.rs`. no engine commands or document fields were added.

- the preview and histogram use the existing proxy, capped at 900 px on the longest side. each
  pixel with positive finite alpha counts once; transparent and non-finite pixels are excluded,
  finite values outside 0–1 count in the edge bins and the overflow counters.
- the vectorscope converts at most 8192 pixels, on a 2-d grid, through CMS. it is cached per
  preview revision, before/after and selected-region view.
- pointer hover samples one proxy pixel and does at most one ICC conversion; it never recounts
  the histogram or rebuilds the vectorscope.
- a feathered selection blends the preview with its proxy coverage, as the engine does on ok.
  thin details and masks are approximate at proxy size.
- point curves in new settings (dialog, control channel, `filter.cameraRaw`) are validated:
  empty, or 2–16 finite points in 0–255 with inputs at least one level apart. stored smart
  filters are not validated on load, so curves saved by earlier versions keep rendering.

`camera_raw_bench` times the CPU path on a 36 mp layer (P31: develop + histogram; P36: all SDR
scopes, probes, hover and tessellation). GPU upload and presentation are not included.

## not covered

- HDR histograms, output and display: no suitable HDR hardware was available for validation.
  float input and counters outside 0–1 do not imply HDR support.
- PSD camera raw settings outside the verified develop subset described above, and
  pixel-identical adobe camera raw rendering (see the file-compatibility scorecard).
- camera raw's own mask authoring; scopes use the document selection.

references: adobe, make color and tonal adjustments,
julieanne kost, camera raw shortcuts and interactions.
only public behaviour was consulted; no adobe code, shaders, profiles or assets are included.
