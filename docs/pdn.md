# paint.NET documents

brahma opens PDN3 `.pdn` files natively, without paint.NET, python, .NET or an external
converter. file › open, recent files, drops, the CLI and automation use the shared importer.
the browser file picker also accepts `.pdn`.

each bitmap layer remains editable. layer order, unicode names, visibility, opacity and pixel
alpha are retained. painting, masks, transforms, reordering, duplication and undo use the same
commands as other pixel layers. all 14 PDN3 blend modes are supported on the CPU and GPU.
reflect, glow, negation and XOR have no photoshop equivalent, and color burn and color dodge have
separate paint.NET variants because their edge cases differ. the blend menus stay photoshop's: a
layer that uses one of these six modes lists its own mode after photoshop's, and other layers don't
show them.

PDN support is **import only**. save opens save as and suggests `.pcraft`, which preserves the
layers and every blend mode. exporting as `.pdn` fails with an explanatory error. PSD cannot
represent six paint.NET variants; exporting those as PSD reports that their blending was
changed to normal. use `.pcraft` for continued editing, and PNG for a flattened rendering.

the reader supports PDN3's typed NRBF objects, both numeric and older blend-operation records,
BGRA32 surfaces with padded rows, shared pixel memory, and gzip or raw numbered pixel chunks.
it validates references, dimensions, strides, counts, chunk ranges and gzip checksums. metadata
object nesting is capped at 64, layers at 1024 and total decoded pixel memory at 1 gib. imports
can be cancelled while parsing, decompressing or copying pixels. unsupported layouts and
versions return errors. paint.NET document metadata (including print resolution and profiles)
is currently omitted; nonempty user metadata produces an import warning. layer data is preserved.

## verification

synthetic unit tests exercise every blend mode, names, hidden layers, opacity, alpha, row
padding, reversed chunks, old/new mode records, native saves, cancellation and malformed data.
the GPU parity tests include the additional modes. local real-file checks are opt-in:

```sh
PDN_TEST_DIR=/path/to/pdn-and-png-exports cargo test -p brahma-io --test pdn_corpus -- --ignored --nocapture
PDN_TEST_FILE=/path/to/layered.pdn cargo test -p brahma-engine --test pdn -- --ignored
```

each PDN in `PDN_TEST_DIR` needs a corresponding `.png` export with the same basename. the
test checks pixels against that export and verifies layer preservation through `.pcraft`.
keep real-file corpora outside the application repository.

the local paint.NET 4.0.10 corpus (14 small texture documents, 1–8 layers) also matches its
embedded full-size previews within 2.142/255. older paint.NET rounded between layers; brahma
keeps floating-point precision, so layered composites are not necessarily byte-identical.
a supplied 5.x document (1280×720, 11 layers, header version `5.3.8488.42200`) passes editing,
painting, undo/redo and native-save checks. its embedded 256×144 preview matches a catmull–rom
downsample of our rendering with mean error 0.418/255 and maximum error 8/255. its neighboring
PNG differs from the saved PDN preview, so it is unsuitable as a full-size rendering oracle.
these checks do not establish compatibility with every paint.NET release.

format references: ms-NRBF
and the pypdn reader.
this reader implements the layout independently; it does not run .NET deserialization.
