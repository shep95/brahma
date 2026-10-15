# macos DMG window

what finder shows when the DMG opens: a 660 × 400 pt background with the app icon and the
`Applications` link side by side. `package.sh` copies these files into the image; nothing here is
generated at build time, so the DMG still builds with `hdiutil makehybrid` (no mounted device, no
finder scripting on ci, nothing extra installed in the signing job).

| file | what |
|---|---|
| `background.svg` | source of the background: the app icon (`assets/app-icon/brahma-small.svg`, linked, not copied) cropped as a cover on the brahma colour field (`#2f7bf5`), ink and paper. its text (inter, JetBrains mono) is outlined, so rendering it needs no fonts. |
| `background.tiff` | the background at 1x (660 × 400 px, 72 dpi) and 2x (1320 × 800 px, 144 dpi) in one hidpi TIFF (16-colour palette, deflate, srgb; about 90 kb). goes to `.background/background.tiff`. |
| `dmg-layout.DS_Store` | finder's view settings for the volume: window size, icon size 128, brahma.app at (326, 205), `Applications` at (574, 205), and the background. goes to `.DS_Store` in the image; named so it isn't mistaken for (or ignored like) a finder-generated `.DS_Store`. |
| `generate.py` | writes `background.tiff` and `dmg-layout.DS_Store` from the SVG and the layout above. |

## the volume name has no version

the mounted volume is called `brahma`, not `brahma <version>`. the layout points at the background
through an alias that includes the volume name, and finder resolves it by that name: with a
versioned name the window keeps its size and icon positions but shows no background (tested).
the DMG file name (`brahma-<version>-macos-<arch>.dmg`) still carries the version.

## rules

- **finder draws the icon labels in black in light and dark mode** when a window has a background,
  so the area under both icons stays light (paper).
- **nothing goes inside the icon boxes:** artwork keeps 10 pt clear of each 128 pt icon box and of
  the label strip under it.

## regenerate

edit `background.svg` (or the layout constants in `generate.py`), then run, on any os:

```sh
pip install 'pillow>=12' ds_store==1.3.3 mac_alias==2.2.3
python3 packaging/macos/dmg/generate.py   # needs resvg on PATH
```

both outputs are byte-for-byte reproducible and carry nothing from the machine that ran it: the
SVG renders without any fonts, and `dmg-layout.DS_Store` is written from scratch, its background
alias holding only the volume name and `/.background/background.tiff`. if you add text to the SVG,
outline it (`usvg` from resvg converts text to paths). the window is 660 × 432 with finder's 32 pt
title bar; the content area is 660 × 400.

this follows #houseofasher's `packaging/macos/dmg/` (storytold/houseofasher#493).
