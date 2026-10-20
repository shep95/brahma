#!/usr/bin/env python3
"""Render brahma's icon scene: a lone figure in a wind-blurred golden field under a pale
dawn sky, as a 1024 px square. Drawn from colour and composition notes of the reference
picture (sky #c4cfd0 → #ead9c5, field #d9bd93 → #b3834f, the figure at 0.505 × 0.735), so
no photograph is bundled. Feed the result to icon-from-image.py for the rounded icon set.

    packaging/icon-field-scene.py <out.png>
"""
import sys
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

S = 1024
y = np.linspace(0.0, 1.0, S)[:, None]
x = np.linspace(0.0, 1.0, S)[None, :]
rng = np.random.default_rng(7)

def lerp(a, b, t):
    a, b = np.array(a, float), np.array(b, float)
    return a[None, None, :] * (1 - t)[..., None] + b[None, None, :] * t[..., None]

horizon = 0.70
# Sky: cool grey-blue at the top warming to peach haze at the horizon.
t_sky = np.clip(y / horizon, 0, 1) ** 1.3 * np.ones_like(x)
sky = lerp((196, 207, 208), (236, 219, 199), t_sky)
# A warm glow band just above the horizon.
glow = np.exp(-((y - horizon + 0.04) / 0.07) ** 2) * np.ones_like(x)
sky = sky * (1 - glow[..., None] * 0.35) + np.array([242, 214, 184])[None, None, :] * glow[..., None] * 0.35
# Field: pale gold near the horizon, deeper and darker at the bottom, with wind streaks.
t_field = np.clip((y - horizon) / (1 - horizon), 0, 1) * np.ones_like(x)
field = lerp((218, 191, 148), (170, 122, 74), t_field ** 0.8)
streaks = rng.normal(0, 1, (S, S))
streaks = np.array(Image.fromarray(((streaks - streaks.min()) / np.ptp(streaks) * 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(radius=3)).resize((S, S)), float)
streaks = (streaks - streaks.mean()) / (streaks.std() + 1e-6)
# Stretch the texture horizontally so it reads as blown grass.
streaks = np.array(Image.fromarray(np.clip(streaks * 40 + 128, 0, 255).astype(np.uint8)).resize((S // 6, S)).resize((S, S), Image.BILINEAR), float) - 128
field = np.clip(field + streaks[..., None] * 0.22 * t_field[..., None] ** 0.5, 0, 255)
# Soft horizon: blend the two across a haze band.
mix = 1 / (1 + np.exp(-(y - horizon) / 0.012)) * np.ones_like(x)
img = sky * (1 - mix[..., None]) + field * mix[..., None]
img = Image.fromarray(np.clip(img, 0, 255).astype(np.uint8), "RGB")
# The figure: a small dark silhouette standing just past the horizon.
fig = Image.new("L", (S, S), 0)
d = ImageDraw.Draw(fig)
cx, feet, h = int(S * 0.505), int(S * 0.735), int(S * 0.115)
w = int(h * 0.26)
head = int(h * 0.16)
d.ellipse((cx - head // 2, feet - h, cx + head // 2, feet - h + head), fill=255)
d.rounded_rectangle((cx - w // 2, feet - h + head - 2, cx + w // 2, feet - int(h * 0.42)), radius=w // 3, fill=255)
d.rectangle((cx - w // 2 + 2, feet - int(h * 0.45), cx - 2, feet), fill=255)
d.rectangle((cx + 2, feet - int(h * 0.45), cx + w // 2 - 2, feet), fill=255)
fig = fig.filter(ImageFilter.GaussianBlur(radius=1.6))
img.paste(Image.new("RGB", (S, S), (44, 36, 32)), (0, 0), fig)
# The whole picture carries the reference's soft focus.
img = img.filter(ImageFilter.GaussianBlur(radius=0.8))
img.save(sys.argv[1])
print("wrote", sys.argv[1])
