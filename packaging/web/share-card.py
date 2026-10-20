#!/usr/bin/env python3
"""Compose the link-preview picture (Open Graph / Twitter card) for the web app.

    packaging/web/share-card.py <app screenshot png> <out png>

A 1200 × 630 card after the owner's reference design: a deep forest-green field with soft
light, the pitch on the left ("Create Without Limits" and four points), and the app screenshot
in a rounded window on the right. Fonts: Inter from assets/fonts. The screenshot comes from the
`snapshot` example with a public-domain painting open, so nothing in the card is a photograph.
"""
import sys
import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

W, H = 1200, 630
ROOT = __file__.rsplit('/packaging/', 1)[0]
F = ROOT + '/assets/fonts/'

def font(name, size):
    return ImageFont.truetype(F + name, size)

shot = Image.open(sys.argv[1]).convert('RGB')
# Background: a dark green field, lighter towards the top right, with blurred bokeh.
y = np.linspace(0, 1, H)[:, None]
x = np.linspace(0, 1, W)[None, :]
base = np.zeros((H, W, 3))
base[..., 0] = 10 + 18 * (1 - y) * x
base[..., 1] = 16 + 34 * (1 - y) * x + 6 * (1 - y)
base[..., 2] = 9 + 12 * (1 - y) * x
bg = Image.fromarray(np.clip(base, 0, 255).astype(np.uint8))
rng = np.random.default_rng(3)
bokeh = Image.new('RGB', (W, H), (0, 0, 0))
bd = ImageDraw.Draw(bokeh)
for _ in range(26):
    cx, cy, r = rng.integers(0, W), rng.integers(0, H), int(rng.integers(18, 70))
    g = int(rng.integers(40, 110))
    bd.ellipse((cx - r, cy - r, cx + r, cy + r), fill=(g // 3, g, g // 4))
bokeh = bokeh.filter(ImageFilter.GaussianBlur(26))
bg = Image.blend(bg, Image.fromarray(np.clip(np.array(bg, int) + np.array(bokeh, int) * 0.55, 0, 255).astype(np.uint8)), 1.0)
# A beam of light from the top right.
beam = Image.new('L', (W, H), 0)
ImageDraw.Draw(beam).polygon([(W, 0), (W - 420, 0), (W - 150, H), (W, H)], fill=70)
beam = beam.filter(ImageFilter.GaussianBlur(60))
bg = Image.composite(Image.new('RGB', (W, H), (90, 110, 60)), bg, beam)
card = bg.convert('RGBA')

# The pitch.
d = ImageDraw.Draw(card)
cream, dim, gold = (236, 232, 222), (170, 168, 160), (201, 168, 89)
# Brand mark: a small triangle like the reference, in gold.
d.polygon([(60, 86), (84, 44), (108, 86)], outline=gold, width=4)
d.text((60, 104), 'E S S E N T I A L S', font=font('Inter-Medium.ttf', 15), fill=dim)
d.text((58, 150), 'Create', font=font('Inter-SemiBold.ttf', 62), fill=cream)
d.text((58, 216), 'Without', font=font('Inter-SemiBold.ttf', 62), fill=cream)
d.text((58, 282), 'Limits', font=font('Inter-SemiBold.ttf', 62), fill=gold)
d.text((60, 368), 'a modern image editor', font=font('Inter-Regular.ttf', 21), fill=cream)
d.text((60, 396), 'for creators, designers', font=font('Inter-Regular.ttf', 21), fill=cream)
d.text((60, 424), 'and visual thinkers.', font=font('Inter-Regular.ttf', 21), fill=cream)
points = [('powerful tools', 'edit with precision'), ('clean interface', 'stay in flow'), ('creative freedom', 'bring your ideas to life'), ('built for creators', 'fast, flexible, and modern')]
for i, (a, b) in enumerate(points):
    yy = 466 + i * 40
    d.rounded_rectangle((60, yy, 88, yy + 28), radius=7, outline=(120, 118, 110), width=2)
    d.text((100, yy - 1), a, font=font('Inter-Medium.ttf', 16), fill=cream)
    d.text((100, yy + 15), b, font=font('Inter-Regular.ttf', 12), fill=dim)

# The app window: scaled, rounded, with a shadow.
tw = 790
th = int(shot.height * tw / shot.width)
win = shot.resize((tw, th), Image.LANCZOS)
wx, wy = W - tw - 10, 54
shadow = Image.new('RGBA', (W, H), (0, 0, 0, 0))
ImageDraw.Draw(shadow).rounded_rectangle((wx - 6, wy + 10, wx + tw + 6, wy + th + 26), radius=14, fill=(0, 0, 0, 170))
shadow = shadow.filter(ImageFilter.GaussianBlur(18))
card = Image.alpha_composite(card, shadow)
mask = Image.new('L', (tw * 3, th * 3), 0)
ImageDraw.Draw(mask).rounded_rectangle((0, 0, tw * 3 - 1, th * 3 - 1), radius=36, fill=255)
mask = mask.resize((tw, th), Image.LANCZOS)
card.paste(win, (wx, wy), mask)
ImageDraw.Draw(card).rounded_rectangle((wx, wy, wx + tw, wy + th), radius=12, outline=(70, 72, 60, 255), width=1)
card.convert('RGB').save(sys.argv[2], quality=90)
print('wrote', sys.argv[2])
