# brahma app icon

the project owner's own picture (a kneeling figure against the light), centre-cropped to a
rounded square (corner radius 22 % of the side). `packaging/icon-from-image.py <picture>` crops,
rounds and writes every size here plus the web favicon next to `apps/brahma-web/index.html`;
`apps/brahma-web/icon-{192,512}.png` are the 1024 master resized, and `icon-maskable-512.png`
is the square picture on its own edge colour with the pwa safe-zone margin.

to change the icon: `python3 packaging/icon-from-image.py <picture>`, then regenerate the three
pwa icons the same way (see `docs/handoff.md`), and commit the outputs: packaging never needs
pillow.
