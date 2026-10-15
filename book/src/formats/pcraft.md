# the `.pcraft` format

`brahma-format` stores the native lossless document as a ZIP archive or directory bundle. its main entries are:

```text
manifest.json
tiles/<blake3>.zst
blobs/<blake3>.zst
thumb.png
composite/preview.png
```

the manifest is versioned and describes the document tree. tiles and blobs are content-addressed. loaders verify ZIP entry bounds and crcs; document content also uses stored hashes to detect mismatched or corrupted tile/blob content. unsupported newer format versions fail explicitly.

## implemented load limits

default `LoadOptions` apply these maximums:

| limit | default |
|---|---:|
| `manifest.json` | 256 mib |
| one decompressed blob | 1 gib |
| total decompressed tiles and blobs | 16 gib |

layer groups may nest at most 100 levels (`MAX_GROUP_DEPTH`). saving a deeper document fails with an error instead of writing a bundle that can't be opened, and a manifest whose JSON nesting goes beyond what that depth needs is rejected before it is parsed, so a hostile file can't exhaust the stack.

the ZIP reader bounds decompression by the caller-supplied maximum, rejects encrypted entries and unsupported compression methods, validates offsets with checked arithmetic, and rejects CRC or declared-size mismatches. corruption and property tests cover truncation, byte mutation, missing content, invalid manifests, excessive sizes, and random bytes.

these defaults are intentionally finite but can still be expensive on constrained systems. future hardening should add context-sensitive budgets and explicit limits for entry count, document complexity, and cumulative work, then test directory bundles against symlink and replacement races.

when fields are added to `brahma-doc`, `brahma-format` intentionally requires corresponding manifest and conversion updates so native saves do not silently drop new state.
