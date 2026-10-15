# camera raw preview navigation

the camera raw dialog's preview has zoom, hand, fit in view, 100%, and zoom preset
controls at the bottom left.

choose **fit in view**, or press **ctrl+0** on windows/linux (**command+0** on macos),
to fit the whole image inside the preview.

at **100%**, one source pixel maps to one physical display pixel, including hidpi.
the native preview refines the processed source in the background so texture,
sharpening, and grain can be inspected beyond the reduced proxy's resolution.

## controls

| action | windows / linux | macos |
|---|---|---|
| select zoom | z | z |
| scrub zoom in / out | left-drag right / left with zoom | same |
| toggle fit / 100% | click with zoom | same |
| zoom a selected region | ctrl+drag | command+drag |
| select hand | h | h |
| temporary pan | space+drag, or middle-button drag | same |
| pointer-anchored zoom | alt+wheel, or pinch | option+wheel, or pinch |
| zoom in / out | ctrl+plus / minus | command+plus / minus |
| fit in view | ctrl+0 | command+0 |
| actual size | ctrl+alt+0 | command+option+0 |
| zoom presets | bottom-left dropdown or right-click menu | same |

zoom and pan are view state: they do not change filter parameters or document
history. before/after keeps the same position; colour probes and clipping overlays
follow the image coordinates. cancel discards the filter; ok applies it through
the existing camera raw command.

## preview limits

the fast proxy remains available while native refinement is pending. processed
full-resolution refinement is limited to 64 megapixels because the existing
engine allocates full-image temporary buffers. larger sources and failed
refinement retain the proxy with a notification. only the visible crop is uploaded
to the GPU (at most 16,777,216 pixels and the device texture-size limit).
the initial preview domain is bounded to 512 megapixels, including sparse content
outside the canvas. histogram and vectorscope statistics remain proxy-based.
WebAssembly retains the processed proxy until a browser worker path is available.

gesture references: adobe camera raw introduction
and julieanne kost's camera raw shortcuts.
