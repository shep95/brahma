# hosting brahma for the web

`brahma-web-<version>.zip` (from the GitHub release, or `packaging/web/package.sh`) holds a
static site in `brahma-web-<version>/`:

| file | what it is |
|---|---|
| `index.html` | the page. it loads everything through relative urls. |
| `brahma-web-<hash>.js` | wasm-bindgen glue (generated, es module) |
| `brahma-web-<hash>_bg.wasm` | the app: about 23 mib raw, 10.5 mib with gzip (see [sizes](#sizes)) |
| `_headers`, `.htaccess` | sample header rules for netlify/cloudflare pages and apache |
| `fonts/` (optional) | fonts loaded on demand, listed in `fonts/manifest.txt` (see [fonts](#fonts)) |

there is no server-side code. upload the folder's contents anywhere that serves static files.

## vercel

the repository deploys to vercel as a static site with no extra setup:
`vercel.json` at the root sets the build command to `packaging/web/vercel-build.sh`, the output
directory to `dist/web`, and the header rules below (wasm MIME type, immutable caching for the
hashed assets, `no-cache` for `index.html`). compression and HTTPS come from vercel itself, so
webgpu and the clipboard work out of the box.

1. import the GitHub repository in the vercel dashboard (**add new… → project**), or run
   `vercel` from the repository root. leave the framework preset as "other"; the settings come
   from `vercel.json`.
2. deploy. the build script installs a stable rust toolchain with the `wasm32-unknown-unknown`
   target and the pinned trunk, then runs the same `trunk build --release` as `package.sh`.
   the first build compiles the whole workspace to wasm and takes a while (tens of minutes);
   vercel's build time limit is 45 minutes.

the build links with thin LTO and two cargo jobs to stay inside the builder's memory, like the
docker build. the wasm is a little larger than the release zip's fat-LTO build. to change this,
set `CARGO_PROFILE_WASM_RELEASE_LTO=fat` and/or `CARGO_BUILD_JOBS` in the project's environment
variables (vercel pro builders with more memory handle fat LTO). the about dialog shows the
deployed commit through `VERCEL_GIT_COMMIT_SHA`. to serve extra fonts, add a `fonts/` folder (see
[fonts](#fonts)) to `apps/brahma-web/` and reference it from `index.html` with a
`<link data-trunk rel="copy-dir" href="fonts" />` tag.

## docker

> the docker recipe (`Dockerfile`, `.dockerignore`, `packaging/web/nginx.conf`) is
> **community-maintained**: it is not built in ci or used for releases, so it can lag behind the
> official web build above. fixes are welcome.

from the repository root (docker is the only build prerequisite):

```sh
docker build --load -t brahma-web:local .
docker run -d --name brahma-web --restart unless-stopped \
  -p 8080:8080 brahma-web:local
```

open **http://localhost:8080/**. the multi-stage `Dockerfile` builds the existing rust/wasm
app with trunk, including HEIF support, then copies only the static site into NGINX. the
runtime runs as the `nginx` user on port **8080**, with a `/healthz` endpoint and docker
health check. it serves precompressed gzip assets, the wasm MIME type, immutable hashed
assets, and an HTML page that revalidates after deployments. no rust installation, node.js,
database, GPU passthrough, or document volume is needed on the host.

the first build downloads rust dependencies and trunk's wasm tools and can take several
minutes. subsequent builds reuse cargo caches. the builder supports linux amd64 and arm64;
the default rust version is the stable release ci used when the `Dockerfile` was last updated
(ci and releases always use the latest stable), and trunk is pinned to the release workflow's
`TRUNK_VERSION`. `RUST_VERSION`, `TRUNK_VERSION`, and `NGINX_VERSION` can be overridden with
`--build-arg`. to stamp the about dialog with the commit and date instead of "dev build", pass
`--build-arg BRAHMA_BUILD_SHA=$(git rev-parse HEAD) --build-arg BRAHMA_BUILD_DATE=$(date -u +%F)`.
the web build does not embed the optional `craft-fonts` checkout.

the docker build uses thin LTO and one cargo build job to reduce peak memory, while keeping
the web profile's size optimizations and `wasm-opt -Oz`. its wasm size may differ from the
release zip figures below; NGINX has no 25 mib file limit. to use the release zip's fat LTO on
a builder with more memory, pass `--build-arg CARGO_PROFILE_WASM_RELEASE_LTO=fat`.
`--build-arg CARGO_BUILD_JOBS=4` enables more concurrent compilation on larger builders. a compiler stopped with
`SIGKILL`/`cannot allocate memory` means the docker/podman vm needs more memory or swap.

for a public deployment, put an HTTPS reverse proxy or your hosting platform's TLS endpoint
in front of container port 8080. with a reverse proxy on the same host, bind only loopback
(`-p 127.0.0.1:8080:8080`); with a proxy container, connect both through a docker network.
HTTPS enables webgpu and browser clipboard apis. if hosting under `/brahma/`, redirect
`/brahma` to `/brahma/` and strip that prefix when proxying to the container. relative
asset urls then work without rebuilding. for example, in an existing HTTPS NGINX server:

```nginx
location = /brahma { return 301 /brahma/; }
location /brahma/ {
    proxy_pass http://127.0.0.1:8080/;
}
```

the editor and image processing run on the visitor's device. open uses the browser file
picker; save and export download files. preferences use browser localStorage. there is no
server-side document storage, desktop TCP control server, or web autosave/crash recovery;
save/download work before closing or reloading the page. a browser with webgpu or webgl2 is
required; `?webgl` forces the fallback when troubleshooting.

to verify the browser deployment, open the page, open an image and save/export it.

## sizes

measured on 2026-10-12 (0.8.0, `packaging/web/package.sh`; gzip `-9`):

| file | raw | gzip |
|---|---|---|
| `brahma-web-<hash>_bg.wasm` | 24,167,186 bytes (23.0 mib) | 10,998,053 (10.5 mib) |
| `brahma-web-<hash>.js` | about 170 kb | about 26 kb |

per-file upload limits: cloudflare pages and workers static assets reject any file over
**25 mib** (26,214,400 bytes). the wasm fits, and `packaging/web/package.sh` fails the build if
it ever grows past 24 mib, so a release can't ship a file a common host refuses. (releases up to
0.2.0 shipped a 25.8 mib wasm, which cloudflare rejected; issue #198.) the size comes from the
`wasm-release` cargo profile (fat LTO, size-optimized code with the pixel and file-decoding
crates kept at speed) plus `wasm-opt -Oz`, and from storing the bundled fonts (brahma's and
egui's) and the translation catalogs compressed, unpacked on first use.

## any path works

all urls in `index.html` are relative (`public_url = "./"` in `apps/brahma-web/Trunk.toml`),
so the site works at a domain root (`https://example.com/`), under a prefix
(`https://example.com/tools/brahma/`) and from a CDN bucket. the asset names carry a content
hash, so they can be cached forever. only `index.html` needs revalidation.

## required server settings

- **MIME type:** serve `.wasm` as `application/wasm`. browsers refuse to stream-compile it under
  any other type, and the app then loads slowly or not at all. serve `.js` as `text/javascript`.
  most hosts already do both. for nginx, check that `mime.types` has `application/wasm wasm;`.
- **compression:** turn on gzip or brotli for `.wasm`, `.js` and `.html`. that takes the
  download from about 19 mib to about 8 mib (gzip) or 5.6 mib (brotli). you can also
  precompress (`brotli -k *.wasm`) and let the server send `Content-Encoding: br`.
- **caching:** `Cache-Control: public, max-age=31536000, immutable` on the hashed `.wasm` and
  `.js` files, and `no-cache` on `index.html`.
- **HTTPS:** webgpu (and the clipboard) only work in a secure context, which means `https://`
  or `http://localhost`. over plain HTTP elsewhere, the app falls back to webgl2.
- **no special isolation headers:** brahma doesn't use `SharedArrayBuffer`, so it doesn't
  need `Cross-Origin-Opener-Policy` or `Cross-Origin-Embedder-Policy`. if your site already sends
  COEP `require-corp`, also send `Cross-Origin-Resource-Policy: same-origin` (or `cross-origin`
  when the files live on a CDN) on the app's files.

nginx example:

```nginx
location /brahma/ {
    types { application/wasm wasm; text/javascript js; text/html html; }
    gzip on;
    gzip_types application/wasm text/javascript text/html;
    location ~* \.(wasm|js)$ { add_header Cache-Control "public, max-age=31536000, immutable"; }
    location ~* index\.html$ { add_header Cache-Control "no-cache"; }
}
```

local test: `python3 -m http.server 8765` inside the folder, then open http://localhost:8765/.

## fonts

the app has two fonts of its own, inter and JetBrains mono: a browser offers no system fonts, and
embedding more would push the wasm over the size gate. a host can serve more fonts next to the
app. they show in the font menus straight away, and a family's files are downloaded the first time
it is picked or a document's text needs it, so starting the app costs one small text file.

1. put the font files (`.ttf`, `.otf`, `.ttc`) anywhere under `fonts/` next to `index.html`.
2. list them in `fonts/manifest.txt`, one file per line, in the format of
   craft-fonts' own `fonts/manifest.txt`:

   ```text
   # family | style | file (relative to the site root) | scripts | licence | licence file | sha256 | source
   Open Sans | Regular | fonts/open-sans/OpenSans[wdth,wght].ttf
   Open Sans | Italic | fonts/open-sans/OpenSans-Italic[wdth,wght].ttf
   Lobster | Regular | fonts/lobster/Lobster-Regular.ttf | Latn,Cyrl | OFL-1.1 | fonts/lobster/OFL.txt
   ```

   the app reads the first three fields; the others are optional here. `family` must be the font's
   own family name (the typographic family in its `name` table, else the family), which is what the
   font menu shows once the file is in. the path is relative to the site root and can't leave it
   (no `..`, no `/` at the start, no other host). a variable font is one line: the style menu offers
   the weights of its `wght` axis.
3. put each font's licence next to it; the SIL open font license, for one, requires that.

a craft-fonts checkout is already laid out this way, and so is any folder with a
`fonts/manifest.txt` and the files it lists: `BRAHMA_WEB_FONTS_DIR=<that folder>
packaging/web/package.sh` copies the manifest, the fonts and their licences into the zip, keeping
their paths. without `fonts/manifest.txt` (a 404) nothing changes.

serving: let `fonts/manifest.txt` revalidate (`Cache-Control: no-cache`) so edits show up; cache
the font files for long only if their names change with their contents. serve `.ttf` as `font/ttf`
and `.otf` as `font/otf`, and compress them (brotli roughly halves a TrueType file). the app
fetches them with `fetch()`, so a content security policy needs them allowed by `connect-src`.

## embedding in a page (iframe)

```html
<iframe
  src="https://example.com/brahma/"
  title="brahma image editor"
  style="width: 100%; height: 720px; border: 0;"
  allow="fullscreen; clipboard-read; clipboard-write"
  allowfullscreen>
</iframe>
```

- the app fills the iframe and follows its size, so size the iframe and not the app.
- keyboard shortcuts go to the iframe after the user clicks into it, as with any embedded app.
- **cross-origin embeds** work. preferences are kept in the iframe's `localStorage`. browsers
  that partition or block third-party storage may forget them between visits, and the app
  then starts with defaults.
- **sandboxed iframes** need at least
  `sandbox="allow-scripts allow-same-origin allow-downloads allow-popups"`. without
  `allow-same-origin` there's no storage. without `allow-downloads`, save and export (browser
  downloads) are blocked.
- don't send `X-Frame-Options: DENY` or a `frame-ancestors` CSP that excludes the embedding page.

## renderer selection and fallback flags

brahma renders with wgpu. it uses **webgpu** when the browser has it and falls back to
**webgl2** on its own. URL query flags override this, and they work on the iframe `src` too:

| flag | effect |
|---|---|
| *(none)* | webgpu if available, otherwise webgl2 |
| `?webgl` | force the webgl2 backend (useful when a webgpu driver misbehaves) |
| `?cpu` | force the CPU canvas path (slowest, most compatible) |

for example: `<iframe src="https://example.com/brahma/?webgl" ...>`.

a browser with neither webgpu nor webgl2 gets a message in place of the app.
