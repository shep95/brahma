# batch output naming

select a recorded action, then open **file > automate > batch**. set the input and output
folders. **file naming** accepts a filename stem template; the selected export format supplies
the extension. leave it blank to keep the existing source filenames and collision reporting.

| token | value |
| --- | --- |
| `{name}` | input filename without its last extension |
| `{serial}` | input position plus **starting serial** (default 1), padded to at least three digits |
| `{date}` | UTC date at the start of the batch, in `YYYY-MM-DD` form |

for example, `{name}_{serial}` with starting serial 7 writes `photo_007.png` and
`photo_008.png` for two PNG inputs named `photo.png`. a serial after 999 becomes 1000;
it does not wrap. folder inputs use the existing filename order, and an explicit input array
keeps its order. failed inputs consume their position, so later filenames do not shift.
braces in an input filename are literal text, not additional tokens.

unknown tokens, unmatched braces, invalid parameter types, path separators, reserved filename
characters, excessive filename length (255 UTF-8 bytes), and serial overflow are rejected before
export writes begin. starting serial must be an integer from 1 to 4,294,967,295.
a custom template that resolves to an existing input file is also rejected; choose a separate
destination or another template. ordinary per-file read, action and save failures still appear
in the batch's `errors` result. when two results share a name, including a case-only difference,
the first successful result is kept and the later collision is reported.

the same options are available on `file.automate.batch` through the engine command interface:

```json
{
  "input": ["first/photo.png", "second/photo.png"],
  "output": "exports",
  "steps": [["image.adjustments.invert", {}]],
  "format": "png",
  "naming": "{name}_{serial}",
  "startingSerial": 7
}
```

the CLI can invoke this command with `run --cmd file.automate.batch --params ...`.
this option belongs to batch; image processor and droplet options retain their existing behavior.
