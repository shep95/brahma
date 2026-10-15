# format overview

external bytes enter brahma through four main paths:

- `brahma-psd` parses and writes PSD and PSB structures.
- `brahma-codecs` decodes and encodes flat raster formats.
- `brahma-format` loads and saves the native `.pcraft` bundle.
- `brahma-affinity` reads native affinity documents (`.af`, `.afdesign`, `.afphoto`, `.afpub`): layers, artboards, vectors, text, images and masks; it does not write them.

`brahma-io` maps PSD and flat images into the document model and exports documents back to those formats. it is the integration layer; parsing limits remain the responsibility of the underlying format crates.

formats are a primary security boundary. file extension is not a trust signal, metadata can be attacker-controlled, and small compressed inputs can declare very large decoded outputs. callers should use the bounded default apis and preserve errors rather than retrying with unlimited limits.

see [PSD and PSB](psd-psb.md), [raster formats](raster-formats.md), [SVG](svg.md), [affinity documents](affinity.md), and [the `.pcraft` format](pcraft.md). cross-format hardening guidance is in [parser hardening](../security/parser-hardening.md).
