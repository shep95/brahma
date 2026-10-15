# affinity documents

brahma opens native affinity documents: `.af` files from affinity 3 and `.afdesign`,
`.afphoto` and `.afpub` files from affinity 1 and 2. layers come in as editable brahma layers:

- pages and artboards become artboards (publisher spreads sit side by side);
- layers and groups become groups;
- curves and shapes (rectangles, ellipses, polygons, stars and the other geometric shapes) become
  shape layers with their fill, gradient and stroke;
- artistic and frame text becomes type layers, drawn with the fonts installed on your computer;
- placed JPEG and PNG images become embedded smart objects, and pixel layers become pixel layers;
- vector and pixel masks, opacity, visibility, locks, names and blend modes are kept.

artboards are recognized from both the legacy flag and current affinity 3 artboard properties.
they keep their names, bounds and editable children when saved as `.pcraft` or PSD; moving an
artboard moves its children together. nested boards become masked groups, and rotated or curved
boards currently use their bounding rectangle.

the document is 8-bit RGB at the affinity document's resolution. what brahma can't reproduce is
listed in a warning when the file opens, for example layer effects, adjustment layers and live
filters, brush strokes (drawn as plain strokes), special shapes such as clouds and hearts (drawn as
ellipses), master pages, and CMYK or lab colour (converted to RGB without the document's profile).
nothing is dropped without a warning.

brahma can't write affinity files. an opened affinity document has no save path: save asks for
a new file (`.pcraft`, PSD, PNG…) and never writes over the affinity file.

if the native data can't be read (a damaged file, or a layout brahma doesn't know), brahma
opens the small PNG preview affinity stores in the file instead, as a single pixel layer named
“affinity preview”, and the warning says why. that preview can't be placed or used by file ▸ place,
stacks, batch processing or other imports that can't show the warning. export PSD or PNG from
affinity in that case.

see [the crate's notes](https://github.com/shep95/brahma/blob/main/crates/affinity/README.md)
for the full list of what is imported, how the reader was written and validated, and its safety
limits.
