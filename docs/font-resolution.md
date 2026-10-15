# PostScript font resolution

PSD text may identify a face by its PostScript name as well as a family/style.
the PostScript prefix need not match the family name, and the family stored with
a run can be a guess made from that name.

## resolution

`FontDb::resolve_postscript` first looks for an installed face with that exact
PostScript name (families that share the guessed prefix are read first), then
falls back to a heuristic split of the name. the heuristic splits at the last
hyphen. a name without a hyphen loses a trailing capitalised style word
(`GmarketSansBold` → gmarket sans, 700; `ExampleSansSemiBoldItalic` → example
sans, 600, italic). a lowercase ending is not a style (`Daylight`), and a style
word after the vendor tag `MT` stays in the family (`ArialRoundedMTBold` →
arial rounded mt bold).

the heuristic also gives the family, weight and slant stored on PSD import when
the exact face is absent. a legacy family whose name ends in a style word and
has no `MT` tag before it (`CooperBlack`) is then stored as the shorter family
(cooper, 900) until the face is installed.

## missing fonts

opening a PSD or affinity file already replaces a run's guessed family with the
installed family its PostScript name resolves to. a guess still stays on a run
when the face was absent at import and the document is reopened from `.pcraft`
after the font is installed, when fonts are registered after opening, or when a
command sets `font` and `postscriptName` together.

missing-font detection (`missing_fonts` and `missing_fonts_in_layer`) and replace
missing fonts therefore use the same decision as layout: a run is missing only
when neither its stored family, its exact PostScript face nor the heuristic
family is installed. a run that resolves is not replaced because an unresolved
run shares its family label.

## cost

detection resolves the PostScript name only for runs whose stored family is not
installed. the first such lookup reads the name tables of installed faces; the
result is cached until fonts are registered or system fonts are loaded, and
layout performs the same lookup for those runs.
