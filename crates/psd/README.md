# brahma-psd

a standalone reader and writer for adobe photoshop **PSD** (version 1) and **PSB** (version 2, "large document format") files. it keeps everything it reads, so an unmodified file writes back byte for byte. it depends on no other brahma crate.

* clean-room implementation from adobe's public *photoshop file formats specification*. where the spec is silent, behavior follows MIT-licensed psd-tools / ag-psd documentation, and the code comments say so.
* `#![forbid(unsafe_code)]`. builds for `wasm32-unknown-unknown`. the core API works on byte slices; the file helpers only exist on native targets.
* parsing never panics on malformed input. limits: dimensions ≤ 300 000; allocations are checked against the remaining input; a single decode produces at most `MAX_DECODED_BYTES` (8 gib on 64-bit targets, 2 gib on 32-bit targets); descriptor nesting depth ≤ 64.

## guarantees

| property | how |
|---|---|
| `PsdFile::from_bytes(b)?.to_bytes()? == b` for unmodified files | channel data keeps its original encoding. unknown blocks and resources are stored raw. exact padding is recorded wherever it can vary: tagged blocks, layer info, section trailers. |
| `from_bytes(to_bytes(f)) == f` for any model `f` | canonical defaults: `padding: None` means "zero-pad to even", and the parser yields `None` whenever the file used that default. |
| deferred decoding | the parser only copies encoded bytes. channels are decoded when asked (`ChannelData::decode`, `ImageData::decode`). |
| no compositing | `PsdBuilder` takes the merged image from the caller. without one, it writes a white placeholder and sets `has_real_merged_data = false` in resource 1057. |

## public API (overview)

```rust
// Top level
PsdFile::from_bytes(&[u8]) -> Result<PsdFile>
PsdFile::to_bytes(&self) -> Result<Vec<u8>>
PsdFile::open(path) / save(path)            // native only
PsdFile { header, color_mode_data, resources, layer_info, layer_info_placement,
          global_layer_mask, global_blocks, layer_mask_trailing, image_data }
PsdFile::layers() -> &[LayerRecord]         // file order = bottom-most first
PsdFile::layers_mut() -> &mut Vec<LayerRecord>
PsdFile::layer(i) -> Option<Layer<'_>>;  iter_layers()
PsdFile::layer_tree() -> Vec<LayerNode>     // nested groups, children bottom-to-top
PsdFile::layer_tree_max_depth(n) -> Vec<LayerNode> // same, groups nested n deep keep no children
PsdFile::decode_merged() -> Result<Vec<u8>> // planar big-endian samples
PsdFile::composite_rgba8() -> Result<RgbaImage>
PsdFile::resource(id), global_block(key), icc_profile(), resolution(),
        has_real_merged_data(), merged_has_alpha(), validate()

// Header
Header { version: Version::{Psd,Psb}, channels, width, height, depth, color_mode, reserved }
ColorMode::{Bitmap, Grayscale, Indexed, Rgb, Cmyk, Multichannel, Duotone, Lab, Unknown(u16)}

// Layers
LayerInfo { merged_alpha, layers: Vec<LayerRecord>, padding }
LayerRecord { rect, channels: Vec<ChannelData>, blend_mode, opacity, clipping, flags,
              filler, mask: MaskData, blending_ranges, name, blocks, extra_trailing }
LayerRecord::name()            // prefers `luni`
LayerRecord::section_divider(), section_type(), layer_id(), fill_opacity(), is_visible(),
             block(key), block_mut(key), channel(id), channel_rect(id), decode_channel(id, depth, version)
ChannelData { id, compression: Option<Compression>, data }   // encoded bytes kept verbatim
ChannelData::encode(id, compression, decoded, w, h, depth, version)
ChannelData::decode(w, h, depth, version); set_decoded(...)
MaskData::{None, Mask(LayerMask), Raw(Vec<u8>)}
LayerMask { rect, default_color, flags, parameters: Option<MaskParameters>, real: Option<RealMask>, trailing, real_first }
BlendMode (27 modes + PassThrough + Unknown([u8;4])); BlendMode::key()/from_key()
Layer<'_>::rgba8() -> Result<RgbaImage>; user_mask() -> Option<Result<GrayImage>>; channel_bytes(id)

// Tagged blocks
TaggedBlock { signature /* 8BIM | 8B64 */, key, data, padding: Option<Vec<u8>> }
TaggedBlock::parsed() -> Option<Result<BlockData>>
BlockData::{UnicodeName, SectionDivider, LayerId, NameSource, BlendClippedAsGroup,
            BlendInteriorElements, Knockout, Protection, SheetColor, FillOpacity, MetadataSetting}
constructors: unicode_name, section_divider, layer_id, name_source, blend_clipped_as_group,
              blend_interior_elements, knockout, protection, sheet_color, fill_opacity
TaggedBlock::check_structure() -> Result<()>   // strict inner re-parse of PlLd, SoLd, SoLE, lnk2/3/D, lfx2
tagged::uses_long_length(version, key)   // PSB 8-byte length keys (the spec's 13 + lnk3, lnkE, pths, extd, extn, FELS, cinf, artd)
LayerInfo::unpadded_len(version), LayerInfo::pad_to(version, 4)   // Photoshop pads the layer info to 4

// Image resources
ImageResource { signature, id, name, data };  ImageResource::parsed() -> Option<Result<ResourceData>>
ResourceData::{ResolutionInfo, LayerState, LayerGroupInfo, Thumbnail, GlobalAngle, IccProfile,
               GlobalAltitude, VersionInfo, Exif, Xmp}

// Compression
Compression::{Raw, Rle, Zip, ZipPrediction, Unknown(u16)}
compression::{encode_planes, decode_planes, PlaneLayout, packbits::{encode, decode}, predict, unpredict}
pixels::{samples_u16, samples_f32, u16_to_bytes, f32_to_bytes, unpack_bits, plane_to_u8}

// Descriptors
descriptor::{Descriptor, VersionedDescriptor, Value, Id, Class, ReferenceItem, UnicodeString, ObjectArray}
Descriptor::from_bytes / to_bytes;  VersionedDescriptor::from_bytes / to_bytes (version 16)

// Builder
PsdBuilder::new(w, h).color_mode(..).depth(8|16).version(..).compression(..).resolution(dpi).icc_profile(..)
builder.push_layer(LayerSpec) / begin_group(GroupSpec) / end_group()? / composite(PixelData)
builder.build() -> Result<PsdFile>;  builder.to_bytes()
PixelData::{Rgba8, Rgba16, GrayA8, GrayA16, Cmyka8, Cmyka16}   // CMYK given as ink, stored inverted

// Test generator (feature `testgen`)
testgen::{all_cases, merged_only, layered, small, sample_descriptor, pattern_plane}
```

## feature matrix

| area | read | write | typed | notes |
|---|:-:|:-:|:-:|---|
| header, PSD v1 / PSB v2 | ✓ | ✓ | ✓ | depth 1/8/16/32; all 8 color modes plus `Unknown` |
| color mode data | ✓ | ✓ | raw | indexed palette is used by `composite_rgba8` |
| image resources | ✓ | ✓ | partial | 1005, 1024, 1026, 1033/1036 (raw), 1037, 1039, 1049, 1057, 1058, 1060 |
| layer records | ✓ | ✓ | ✓ | flags, clipping, filler, blending ranges (raw + accessor) |
| layer masks | ✓ | ✓ | ✓ | 20 / 36-byte forms, mask parameters, real mask; unparseable masks stay raw |
| channel ids -1 / -2 / -3 | ✓ | ✓ | ✓ | per-channel rect via `channel_rect` |
| raw / RLE / ZIP / ZIP+prediction | ✓ | ✓ | – | prediction at 8/16/32 bits (32: byte-plane shuffle + delta); RLE counts are u16 in PSD, u32 in PSB |
| merged image data | ✓ | ✓ | – | validated at parse time (sizes, RLE counts, zlib stream incl. checksum); a zlib stream declaring more than `MAX_DECODED_BYTES` is kept verbatim without inflating it, and decoding it reports the limit |
| tagged blocks (raw passthrough) | ✓ | ✓ | – | keeps `8BIM` / `8B64` and exact padding |
| PSB 8-byte-length keys | ✓ | ✓ | – | lmsk lr16 lr32 layr mt16 mt32 mtrn alph fmsk lnk2 feid fxid pxsd |
| lr16 / lr32 / layr layer info | ✓ | ✓ | ✓ | lifted into `layer_info`; re-inserted at its original position |
| luni lsct lsdk lyid lnsr clbl infx knko lspf lclr iOpa shmd | ✓ | ✓ | ✓ | `TaggedBlock::parsed()` |
| ActionDescriptor (all ostypes) | ✓ | ✓ | ✓ | including references, `ObAr` (best effort), `UnFl`, `Pth `, version-16 wrapper |
| layer tree (groups) | ✓ | ✓ | ✓ | open/closed folders plus bounding dividers; tolerant of malformed nesting |
| RGBA8 extraction | ✓ | – | – | RGB / gray / CMYK layers; merged image also supports indexed / bitmap / duotone |
| builder | – | ✓ | – | RGB / gray / CMYK at 8/16 bit, masks, groups, blend modes, opacity, fill, visibility, clipping |
| patterns | ✓ | ✓ | ✓ | `Patt` / `Pat2` / `Pat3`, `.pat`, and `.abr` embedded patterns; non-empty tiles allow in-bounds channel subrectangles, decoded size is capped and PackBits expansion is input-bounded |
| actions (`.atn`) | ✓ | ✓ | ✓ | `atn::parse` / `atn::write`: version-16 sets (photoshop 6+): actions with function key, modifiers and colour; steps with event code or string name, panel name, enabled/dialog flags and their descriptor. exact round trip; version-12 (photoshop 5) files are rejected as unsupported |

pattern channel subrectangles are padded with zeroes to the tile rectangle. all channel
metadata and the total padded size are checked against `MAX_DECODED_BYTES` before any
plane is decoded. fully populated channels reuse their decoded buffer without a second
copy; allocations for padded channels return an error on failure. there is no additional
padding-specific size limit.

### preserved raw, not yet typed (parsers planned for m8)

* layer effects: `lfx2`, `lrFX`, `lmfx`. the descriptors inside can be parsed on demand with `VersionedDescriptor::from_bytes(&block.data[4..])`.
* type layers: `TySh`, `tySh`, and the EngineData inside them.
* smart objects: `SoLd`, `PlLd`, `SoLE`, `lnk2`, `lnkD`, `lnk3`.
* vector masks and shapes: `vmsk`, `vsms`, `vogk`, `vscg`.
* adjustment and fill layers: `curv`, `levl`, `hue2`, `SoCo`, `GdFl`, `PtFl`, …, plus `Patt`, `Txt2`, `FMsk`, 3d and video blocks.
* global layer mask info (raw, with accessors), thumbnails (raw), slices, guides, paths, and every other resource id.

## testing

`cargo test -p brahma-psd --all-features` runs the unit tests, integration tests and proptests:

* byte stability and model round-trips for every generated case: all depths, color modes and compressions, PSD and PSB.
* PackBits edge cases.
* known vectors for 8/16/32-bit prediction.
* PSD vs PSB length handling.
* truncation sweeps: every prefix of several small files must return `Err`.
* random-mutation and random-byte proptests: never panic.
* layer tree cases.
* builder → `rgba8` correctness.
* unknown blend modes and unknown blocks pass through unchanged.

`tests/corpus.rs` walks `corpus/psd/**/*.{psd,psb}` at the workspace root when that directory exists. for each file it asserts that parsing succeeds and the file round-trips byte for byte, and it prints a result per file. without the directory it skips silently.

a `cargo-fuzz` skeleton lives in `fuzz/`. it is its own workspace and is excluded from brahma's. run it with `cargo +nightly fuzz run parse` (or `descriptor`) from `crates/psd`.

## spec ambiguities and decisions

* **tagged block padding.** the spec says lengths are "rounded up to an even byte count". in practice writers either include the padding in the length or append it after the data, and pad to 2 or 4. the parser detects the padding that actually follows each block: the smallest k ≤ 3 after which the next signature or the end of the region appears. it stores that padding unless it matches the default.
* **layer info padding** (2 vs 4) is recorded the same way.
* **16/32-bit layers** are read from global `Lr16` / `Lr32` blocks when the main layer info is empty. the block data is a layer-info body with no inner length field.
* **mask record layout.** the order is: rect, default color, flags, parameters (when flag bit 4 is set), then the real flags, background and rect whenever at least 18 bytes remain. anything left over (for example the 2 padding bytes of the 20-byte form) is kept in `trailing`.
* **mask channel rects.** channel -2 uses the mask rect. channel -3 uses the real mask rect.
* **merged ZIP data** is a single zlib stream over all planes. prediction is applied per row across all planes.
* **ZIP prediction at depth 1** is treated as byte-wise delta.
* **`ObAr`** is read as a u32 prefix followed by a descriptor-shaped body, following ag-psd's reading. the spec does not document it.
* **descriptor `bool`** bytes other than 0/1 are normalized to 1 on write.
* **pascal names** have their pad bytes written as zeros. a layer & mask section that contains only a zero layer-info length is normalized to an empty section. these are the only known non-byte-exact cases, and both are pathological.
* **header limits.** the parser accepts up to 300 000 × 300 000 for both versions. `PsdFile::validate()` enforces the spec's 30 000 limit for PSD.
