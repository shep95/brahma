#!/usr/bin/env python3
"""Make every brahma app icon and the web favicon from one picture.

    packaging/icon-from-image.py <picture> [--radius 0.22]

The picture is centre-cropped to a square, given rounded corners (the radius as a fraction of
the side, 0.22 by default, like a macOS app tile) and written out as:

  assets/app-icon/brahma-1024.png                     the master, 1024 px with transparent corners
  assets/app-icon/hicolor/<s>x<s>/apps/com.houseofasher.brahma.png   16..512 px (Linux)
  assets/app-icon/brahma.ico                          16..256 px (Windows)
  assets/app-icon/brahma.icns                         16..1024 px (macOS)
  assets/app-icon/brahma.svg                          an SVG wrapper around the 1024 PNG (scalable theme icon)
  apps/brahma-web/favicon.png                         64 px, served next to index.html

Needs Pillow (python3 -m pip install pillow). The outputs are committed, so packaging never needs it.
"""
import sys, os, base64, io
from PIL import Image, ImageDraw

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICON = os.path.join(ROOT, 'assets', 'app-icon')


def rounded_square(img, side, radius_frac):
    img = img.convert('RGBA')
    w, h = img.size
    s = min(w, h)
    img = img.crop(((w - s) // 2, (h - s) // 2, (w - s) // 2 + s, (h - s) // 2 + s)).resize((side, side), Image.LANCZOS)
    # Supersampled mask for clean anti-aliased corners.
    scale = 4
    mask = Image.new('L', (side * scale, side * scale), 0)
    r = int(side * scale * radius_frac)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, side * scale - 1, side * scale - 1), radius=r, fill=255)
    mask = mask.resize((side, side), Image.LANCZOS)
    out = Image.new('RGBA', (side, side), (0, 0, 0, 0))
    out.paste(img, (0, 0), mask)
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    if not args:
        print(__doc__)
        sys.exit(2)
    radius = 0.22
    if '--radius' in sys.argv:
        radius = float(sys.argv[sys.argv.index('--radius') + 1])
    src = Image.open(args[0])
    master = rounded_square(src, 1024, radius)
    os.makedirs(ICON, exist_ok=True)
    master.save(os.path.join(ICON, 'brahma-1024.png'))
    for s in (16, 24, 32, 48, 64, 128, 256, 512):
        d = os.path.join(ICON, 'hicolor', f'{s}x{s}', 'apps')
        os.makedirs(d, exist_ok=True)
        master.resize((s, s), Image.LANCZOS).save(os.path.join(d, 'com.houseofasher.brahma.png'))
    master.save(os.path.join(ICON, 'brahma.ico'), sizes=[(s, s) for s in (16, 20, 24, 32, 40, 48, 64, 128, 256)])
    master.save(os.path.join(ICON, 'brahma.icns'))
    # Scalable theme icon and SVG master: the PNG embedded in an SVG keeps the picture exact.
    buf = io.BytesIO(); master.save(buf, format='PNG')
    svg = ('<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 512 512" width="512" height="512">'
           '<image width="512" height="512" xlink:href="data:image/png;base64,' + base64.b64encode(buf.getvalue()).decode() + '"/></svg>')
    for p in (os.path.join(ICON, 'brahma.svg'), os.path.join(ICON, 'brahma-small.svg'), os.path.join(ICON, 'hicolor', 'scalable', 'apps', 'com.houseofasher.brahma.svg')):
        os.makedirs(os.path.dirname(p), exist_ok=True)
        open(p, 'w').write(svg)
    web = os.path.join(ROOT, 'apps', 'brahma-web')
    master.resize((64, 64), Image.LANCZOS).save(os.path.join(web, 'favicon.png'))
    master.resize((180, 180), Image.LANCZOS).save(os.path.join(web, 'apple-touch-icon.png'))
    print('icons written from', args[0])


if __name__ == '__main__':
    main()
