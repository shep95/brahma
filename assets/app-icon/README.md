# brahma app icon

a lone figure standing in a wind-blurred golden field under a pale dawn sky, in a rounded
square (corner radius 22 % of the side). the picture is rendered by `packaging/icon-field-scene.py`
from colour and composition notes of the project owner's reference photograph, so no photograph
is bundled; `packaging/icon-from-image.py` crops, rounds and writes every size here, plus the web
favicon and the pwa icons next to `apps/brahma-web/index.html`.

to use the original photograph instead: `python3 packaging/icon-from-image.py <photo>`, then
regenerate `apps/brahma-web/icon-192.png`, `icon-512.png` and `icon-maskable-512.png` from the
1024 master and the uncropped square.

files: `brahma-1024.png` (master, transparent corners), `hicolor/<s>x<s>/apps/com.houseofasher.brahma.png`
(linux), `brahma.ico` (windows), `brahma.icns` (macos), `brahma.svg` / `brahma-small.svg` (scalable wrappers).
