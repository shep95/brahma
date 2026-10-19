#!/usr/bin/env python3
"""Distil a research text (the "shepherd — aesthetic research loop" notes) into
`data/corpus.json`: one entry per pattern, domain section or technique, with a title, a
short summary (the first sentences), the domain and keyword tags. The crate embeds the
JSON and searches it (see `src/corpus.rs`).

    python3 crates/aesthetic/scripts/distill.py <research.txt> > crates/aesthetic/data/corpus.json

The script keeps summaries short (a few hundred characters) so the corpus stays small; the
rule base in `src/rules.rs` carries the numbers the recommender actually computes with.
"""
import json, re, sys

TAGS = {
    "color": ["oklch", "oklab", "color", "colour", "hue", "chroma", "lightness", "palette", "gamut", "contrast", "saturation"],
    "type": ["font", "type", "typograph", "serif", "sans", "weight", "opsz", "optical", "line length", "x-height", "variable font", "modular scale"],
    "layout": ["layout", "grid", "spacing", "whitespace", "composition", "alignment", "hierarchy", "proximity", "gestalt", "fitts", "reading pattern", "f-pattern"],
    "motion": ["animation", "easing", "spring", "motion", "transition", "duration", "stagger", "ease-in", "ease-out"],
    "perception": ["perception", "psycholog", "amygdala", "weber", "fechner", "saccade", "fixation", "peripheral", "simultaneous contrast", "halation", "cognitive"],
    "emotion": ["trust", "calm", "urgency", "wonder", "playful", "emotion", "visceral", "behavioral", "reflective"],
    "dark": ["dark mode", "dark background", "halation", "glow"],
    "performance": ["performance", "60fps", "frame", "budget", "jank", "compositor"],
    "accessibility": ["accessib", "wcag", "apca", "reduced motion", "prefers-reduced"],
    "3d": ["shader", "glsl", "webgpu", "pbr", "raymarch", "particle", "three.js"],
    "generative": ["generative", "noise", "fbm", "attractor", "l-system", "flow field", "voronoi"],
}

def tags_for(text):
    low = text.lower()
    return sorted(k for k, words in TAGS.items() if any(w in low for w in words))

def sentences(text, n=3, limit=420):
    text = re.sub(r"\s+", " ", text).strip()
    parts = re.split(r"(?<=[.!?])\s+", text)
    out = " ".join(parts[:n])
    return out[:limit].rstrip() + ("…" if len(out) > limit else "")

def main(path):
    lines = open(path, encoding="utf-8", errors="replace").read().replace("\r\n", "\n").split("\n")
    entries = []
    domain = ""
    i = 0
    while i < len(lines):
        line = lines[i]
        m = re.match(r"^### (?:domain|pattern): (.+)$", line)
        if m:
            domain = m.group(1).strip()
            body = []
            j = i + 1
            while j < len(lines) and not lines[j].startswith("### ") and not lines[j].startswith("## "):
                body.append(lines[j]); j += 1
            text = "\n".join(body)
            # a domain's named sub-patterns: "*1931 — CIE XYZ color space.* mechanism: ..."
            for pm in re.finditer(r"^\*([^*\n]{3,120})\*\s*(.+?)(?=\n\s*\n|\Z)", text, re.M | re.S):
                title = pm.group(1).strip().rstrip(".")
                entries.append({"title": title, "domain": domain, "summary": sentences(pm.group(2)), "tags": tags_for(pm.group(2))})
            # findings and predictions
            for fm in re.finditer(r"\*\*(future prediction \[finding\]|present patterns|the key insight|design implication[^*]*)\:?\*\*:?\s*(.+?)(?=\n\s*\n|\Z)", text, re.S | re.I):
                entries.append({"title": f"{domain}: {fm.group(1).strip()}", "domain": domain, "summary": sentences(fm.group(2)), "tags": tags_for(fm.group(2))})
            i = j
            continue
        m = re.match(r"^NAME: (.+)$", line)
        if m:
            title = m.group(1).strip()
            body = []
            j = i + 1
            while j < len(lines) and not lines[j].startswith("```"):
                body.append(lines[j]); j += 1
            text = "\n".join(body)
            dm = re.search(r"^DOMAIN: (.+)$", text, re.M)
            mech = re.search(r"^MECHANISM:\s*(.+?)(?=^[A-Z ]+:|\Z)", text, re.M | re.S)
            root = re.search(r"^\s*root: (.+)$", text, re.M)
            summary = sentences(mech.group(1) if mech else text)
            if root:
                summary += " root: " + root.group(1).strip()
            entries.append({"title": title, "domain": (dm.group(1).strip() if dm else "technique"), "summary": summary[:520], "tags": tags_for(text)})
            i = j
            continue
        m = re.match(r"^\*\*convergence \d+: (.+?)\*\*$", line)
        if m:
            body = []
            j = i + 1
            while j < len(lines) and not lines[j].startswith("**convergence") and not lines[j].startswith("## "):
                body.append(lines[j]); j += 1
            text = "\n".join(body)
            entries.append({"title": "convergence: " + m.group(1).strip(), "domain": "cross-domain", "summary": sentences(text, 4, 520), "tags": tags_for(text)})
            i = j
            continue
        i += 1
    # de-duplicate by title, keep the first
    seen, out = set(), []
    for e in entries:
        k = e["title"].lower()
        if k in seen or len(e["summary"]) < 40:
            continue
        seen.add(k); out.append(e)
    json.dump(out, sys.stdout, indent=1, ensure_ascii=False)
    sys.stdout.write("\n")

if __name__ == "__main__":
    main(sys.argv[1])
