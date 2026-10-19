#!/usr/bin/env python3
"""Distil the google/fonts repository's METADATA.pb files into data/google_fonts.tsv.

usage: python3 -I google_fonts_catalog.py <dir with one METADATA.pb per family> <out.tsv>

Each METADATA.pb needs a trailing `dir: "ofl/<slug>"` line (the fetch script adds it). The
catalogue lists, per family: name, category, directory in the repository, the files to fetch
(the variable fonts when the family has them, else regular/italic/bold/bold-italic), and the
fallback scripts (ISO 15924) the family's subsets cover. No font bytes are copied: brahma
fetches a file from the pinned commit the first time the family is used.
"""
import os
import re
import sys

SUBSET_SCRIPT = {
    "arabic": "Arab", "hebrew": "Hebr", "devanagari": "Deva", "thai": "Thai", "khmer": "Khmr",
    "lao": "Laoo", "myanmar": "Mymr", "japanese": "Jpan", "chinese-simplified": "Hans",
    "chinese-traditional": "Hant", "chinese-hongkong": "Hant", "korean": "Kore",
}
# One family per script goes first in the catalogue, so it is that script's fallback.
PREFERRED = ["Noto Sans Arabic", "Noto Sans Hebrew", "Noto Sans Devanagari", "Noto Sans Thai", "Noto Sans Khmer",
             "Noto Sans Lao", "Noto Sans Myanmar", "Noto Sans JP", "Noto Sans SC", "Noto Sans TC", "Noto Sans KR"]


def field(s, name):
    m = re.search(r"^\s*%s: \"(.*?)\"$" % name, s, re.M)
    return m.group(1) if m else ""


def main(src, out):
    rows = {}
    for f in sorted(os.listdir(src)):
        if not f.endswith(".pb"):
            continue
        s = open(os.path.join(src, f), encoding="utf-8", errors="replace").read()
        name, category, d = field(s, "name"), field(s, "category"), field(s, "dir")
        if not (name and d) or any(c in name + d for c in "\t\n"):
            continue
        fonts = []
        for block in re.findall(r"^fonts \{(.*?)^\}", s, re.M | re.S):
            fn = field(block, "filename")
            style = field(block, "style")
            w = re.search(r"^\s*weight: (\d+)", block, re.M)
            fonts.append((fn, style, int(w.group(1)) if w else 400))
        if not fonts:
            continue
        variable = [fn for fn, _, _ in fonts if "[" in fn]
        if variable:
            files = variable
        else:
            picked = []
            for want_style, want_w in (("normal", 400), ("italic", 400), ("normal", 700), ("italic", 700)):
                best = min(((abs(w - want_w), fn) for fn, st, w in fonts if st == want_style), default=None)
                if best and best[1] not in picked:
                    picked.append(best[1])
            files = picked or [fonts[0][0]]
        subsets = re.findall(r'^subsets: "(.*?)"$', s, re.M)
        scripts = sorted({SUBSET_SCRIPT[x] for x in subsets if x in SUBSET_SCRIPT})
        rows[name] = (name, category.lower().replace("_", "-"), d, "|".join(files), "|".join(scripts))
    ordered = [rows[n] for n in PREFERRED if n in rows] + [rows[n] for n in sorted(rows, key=str.lower) if n not in PREFERRED]
    with open(out, "w", encoding="utf-8") as fh:
        fh.write("# google fonts catalogue: family\tcategory\tdir\tfiles (|-separated; variable font names hold commas)\tscripts (see scripts/google_fonts_catalog.py)\n")
        for r in ordered:
            fh.write("\t".join(r) + "\n")
    print(f"{len(ordered)} families -> {out}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
