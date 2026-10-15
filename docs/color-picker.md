# color selection and screen sampling

click the color swatch in the type options bar or character panel to open the full color picker. it offers the color field, component strip, HSB/RGB/lab/CMYK and hex fields, and current/new swatches. selecting characters targets that range; with no character selection, the active text layer is targeted. **ok** applies through `type.setStyle`; **cancel** leaves the text and history unchanged. the foreground color is independent of this dialog.

the existing canvas eyedropper continues to sample the document while the color picker is open, including canvas zoom and pan. this works without operating-system screen-capture permission.

## pick a color outside brahma

**pick screen color** is available in full color pickers and compact color-selection popups, including fills, shapes, gradients, layer effects, guides and preferences.

on X11, windows and macos, clicking it takes one snapshot of the connected screens and opens a sampling viewport on each monitor. move the crosshair over a pixel, use the wheel to change the 11 × 11 pixel loupe from 4× to 24×, then click to select. **esc** cancels. the center pixel has a black/white outline and its sampled hex value is displayed below the loupe.

a full dialog receives the new color for inspection; **ok** is still required to apply it. compact popups retain their normal immediate-edit behavior. existing alpha and additive blending are preserved; float RGB widgets retain their linear RGB storage.

screen snapshots are held in memory, are never written or transmitted by the picker, and are released after selection, cancellation, or a change of document, layer or dialog. the combined capture budget is 64 × 1024 × 1024 pixels (about 67 mp) across at most 16 displays, subject to the GPU's texture-size limit. these limits affect screen sampling only, not document dimensions.

## platform behavior

- **linux/X11:** a native rust X11 client captures the composed desktop. an advertised primary-display ICC profile is converted to srgb through brahma's color-management library. outputs with no published profile use an srgb fallback.
- **linux/wayland:** the desktop's XDG `PickColor` portal owns selection and permission ui and returns srgb. magnification and zoom controls depend on the portal implementation; brahma cannot capture other applications directly under wayland.
- **windows:** native screen capture with monitor-specific fullscreen viewports. captured pixels currently use an srgb fallback; HDR and arbitrary monitor-profile accuracy are not guaranteed.
- **macos:** safe CoreGraphics bindings capture the composed screen and use the snapshot's own ICC profile when available. screen recording permission may be required. the existing macos 11 deployment target is preserved. the window-list capture API is deprecated by apple, so current macos releases require explicit manual validation; migration to modern native sampling must also preserve older-app compatibility.
- **web:** browsers exposing the standard EyeDropper API use the browser's native sampling/magnification ui. browser support and zoom controls vary. unsupported browsers retain document sampling and color fields; external screen sampling is unavailable.

the implementation follows publicly documented photoshop behavior for the full type color picker and outside-image sampling. brahma uses an explicit screen-pick action and frozen snapshots rather than photoshop's press-and-drag gesture.

## verification

unit and ui integration tests cover selected text versus whole-layer color changes, cancel/ok and undo, routing asynchronous picks to their initiating popup, alpha preservation, malformed capture bounds, negative display origins, hidpi mapping and cancellation races. the native capture tests also cover padded pixel layouts and memory budgets.

generate screenshots using synthetic artwork without exposing your desktop:

```sh
cargo run -p brahma-ui-egui --example color_picker_demo -- /tmp/brahma-color-picker
```

for reproducible native X11 capture verification, the explicitly ignored test displays an 8 × 8 synthetic swatch in an isolated xvfb display, captures it in memory and checks the known pixel:

```sh
xvfb-run -a cargo test -p brahma screen_capture_reads_a_known_x11_fixture_pixel -- --ignored --test-threads=1
```

windows, macos, wayland portals, browser activation and mixed-DPI monitor arrangements additionally need manual validation on those environments; mock ui tests do not substitute for it.

references: adobe color selection, XDG color portal, apple image color space, EyeDropper specification.
