# releasing brahma

every push to the `release` branch runs `.github/workflows/release.yml`. the workflow builds
signed installers for macos, windows, linux and the web, plus a freebsd tarball, then creates
or updates a **draft** GitHub release named `brahma v<version>`. nobody sees a draft until a
maintainer publishes it.

this is brahma's implementation of the shared
[release playbook](release-playbook.md). user-facing names say **brahma**. files, binaries
and ids stay lowercase (`brahma-<version>-<platform>-<arch>.<ext>`, `com.houseofasher.brahma`).

## cutting a release

1. **bump the version** on `main`. the only place it lives is `[workspace.package] version`
   in the root `Cargo.toml`:

   ```sh
   cargo xtask version                 # prints the current version, e.g. 0.1.0
   cargo xtask version set 0.2.0       # or 0.2.0-rc.1; updates Cargo.toml and Cargo.lock
   ```

   commit the change (`Cargo.toml` + `Cargo.lock`) through the normal review flow.
2. **merge `main` into `release`** (or fast-forward it) and push. the workflow starts by itself.
3. **wait for the draft.** after about 30 to 45 minutes (notarization is the slow part), the
   releases page has a draft `brahma v0.2.0`, tagged `v0.2.0` on the pushed commit, with
   every artifact and `SHA256SUMS.txt`. the notes are generated from the merged prs.
4. **check it.** download an installer or two and look at the job summaries. any
   `::warning::` there means a signing secret was missing and that artifact is unsigned.
5. **add the scorecard deltas** to the draft's notes: what moved in
   [`docs/scorecard.md`](scorecard.md) since the previous release. compare the summary table
   and the performance table of both tags (`git diff v<previous> v<this> -- docs/scorecard.md`)
   and list each area's done / partial / missing change, the scenarios that newly meet their
   budget or got slower, corpus floors raised, and the change in settings that do nothing.
   quote numbers with the baseline's machine and load average.
6. **publish** the draft in the GitHub ui. publishing creates the `v0.2.0` tag. versions with a
   pre-release suffix (`-rc.1`) are marked as pre-releases.

pushing to `release` again before you publish rebuilds the same draft and replaces its assets.
after the draft is published, the workflow refuses to touch that version again, so bump it first.

**test runs:** *actions → release → run workflow* runs the whole pipeline by hand. the optional
`version` input (such as `0.2.0-rc.1`) overrides `Cargo.toml` for that run only. the jobs apply
it with `cargo xtask version set` before building, so the binaries report it too. the run still
needs the `release` environment, which only the `release` branch can use, so pick that branch
in the dialog.

## what gets built

| platform | artifacts | built on |
|---|---|---|
| macos 11+ (universal: apple silicon + intel) | `brahma-<v>-macos-universal.dmg`, `brahma-cli-<v>-macos-universal.zip` | `macos-15` |
| windows 10+ x64 | `brahma-<v>-windows-x64.msi`, `brahma-<v>-windows-x64-portable.zip` | `windows-latest` |
| windows 10+ x86 (32-bit) | `brahma-<v>-windows-x86.msi`, `brahma-<v>-windows-x86-portable.zip` | `windows-latest` |
| linux x86_64 | `brahma-<v>-linux-x86_64.{AppImage,AppImage.zsync,deb,rpm,tar.gz,flatpak}` | `ubuntu-22.04` (flatpak: `ubuntu-24.04`) |
| linux aarch64 | `brahma-<v>-linux-aarch64.{AppImage,AppImage.zsync,deb,rpm,tar.gz,flatpak}` | `ubuntu-22.04-arm` (flatpak: `ubuntu-24.04-arm`) |
| linux riscv64 (glibc 2.39+) | `brahma-<v>-linux-riscv64.tar.gz` | cross-compiled on `ubuntu-24.04` (ports.ubuntu.com riscv64 libs); CLI smoke-tested under QEMU |
| freebsd 14 x86_64 | `brahma-<v>-freebsd-x86_64.tar.gz` | freebsd 14.3 vm on `ubuntu-latest` |
| web | `brahma-web-<v>.zip` (static site; see [`packaging/web/README.md`](../packaging/web/README.md)) | `ubuntu-latest` |

every binary reports its version, the commit and the build date: `brahma --version`,
`brahma-cli --version`, and *help › about brahma*. ci sets `BRAHMA_BUILD_SHA` and
`BRAHMA_BUILD_DATE`, and `crates/engine/src/build_info.rs` reads them at compile time. a plain
`cargo build` doesn't set them and reports `0.2.0 (dev build)`.

every desktop build job (macos, windows, linux, freebsd) also checks out craft-fonts
at the `craft-fonts-ref` commit in `.github/workflows/release.yml` and builds with `CRAFT_FONTS_DIR` and
`CRAFT_FONTS_REQUIRED=1`, so desktop releases embed its japanese fonts (the web build embeds none: see
`docs/development.md` › fonts) and fail rather than ship without them. the packages carry each font's licence as
`OFL-<family>.txt` (`copy_font_licences` in `packaging/env.sh`; the portable zip on windows;
`Contents/Resources/Licenses` in the macos app). bump the pin deliberately, together with the one
in `ci.yml`. rules: `../craftrules/standards/fonts.md`; build option: `docs/development.md` › fonts.

### macos

`packaging/macos/package.sh` builds `aarch64-apple-darwin` and `x86_64-apple-darwin` with
`MACOSX_DEPLOYMENT_TARGET=11.0`, joins them with `lipo`, and assembles `brahma.app`:

- `Info.plist` is generated from `Info.plist.in`. the bundle id is `com.houseofasher.brahma`.
  the plist sets `LSMinimumSystemVersion` 11.0, `NSHighResolutionCapable`, and document types:
  `.pcraft` (owner), plus PSD/PSB and the image formats brahma reads (alternate, so it never
  takes over preview's defaults). the icon is `assets/app-icon/brahma.icns`.
- **signing** goes inside-out with the hardened runtime and a secure timestamp. the executable
  is signed first, then the bundle. there's no `--deep` on the final signature. the
  entitlements (`entitlements.plist`) are deliberately empty.
- **notarization:** the app is zipped and sent with `xcrun notarytool submit --wait`, then the
  ticket is stapled to the app. the app goes on a DMG (`hdiutil`, with an `Applications` link
  to drag onto). the DMG is signed, notarized and stapled too. the script checks the results
  with `codesign --verify --strict`, `stapler validate` and `spctl -a -vvv`.
  its finder window (background, icon size and positions) comes from
  [`packaging/macos/dmg/`](../packaging/macos/dmg/README.md), and its volume is named `brahma`
  without the version, which the window's background needs; the DMG file name keeps the version.
- **CLI:** the universal `brahma-cli` is signed with the same developer id, the hardened
  runtime and a secure timestamp (identifier `com.houseofasher.brahma-cli`), zipped, and the zip
  is sent to `notarytool`. only `.app`, `.dmg` and `.pkg` can hold a stapled ticket, not a bare
  mach-o, so gatekeeper looks the CLI's ticket up online the first time a downloaded
  (quarantined) copy runs. offline, that first run can be refused until the mac is online again.
- **verification:** `packaging/macos/verify.sh` checks the artifacts as users download them. it
  unpacks the CLI zip and requires, for the binary inside, `codesign --verify --strict`, both
  architectures, the hardened runtime flag, a `Developer ID Application` authority from
  `APPLE_TEAM_ID`, a timestamp, and `spctl --assess --type install` reporting
  `source=Notarized Developer ID` (the same online lookup gatekeeper does). for the DMG it runs
  `codesign --verify`, `stapler validate` and `spctl --type open`. the release workflow runs it
  as its own step after packaging. with signing secrets it fails the job on any miss; without
  them it only checks signature integrity and warns, like `package.sh`. run it locally after a
  build: `packaging/macos/verify.sh --arch aarch64`.

locally, without certificates, the script signs ad-hoc (`codesign -s -`) and skips notarization.
that's enough to check the bundle and the DMG on your own mac:

```sh
packaging/macos/package.sh                    # universal; needs both rustup targets
packaging/macos/package.sh --arch aarch64     # quicker, host-only
open dist/release/brahma-*-macos-*.dmg
```

### windows

`packaging/windows/package.ps1 -Arch x64|x86` builds with `-C target-feature=+crt-static`.
the static c runtime means neither the MSI nor the portable zip needs the visual c++
redistributable, which matters for a standalone installer and costs only about 100 kb. the flag
goes in `CARGO_TARGET_<TRIPLE>_RUSTFLAGS`, so host build scripts aren't affected.

- `apps/brahma/build.rs` and `apps/brahma-cli/build.rs` embed the icon
  (`assets/app-icon/brahma.ico`) and VERSIONINFO with the `winresource` crate
  (build dependency `0.1`). they only do this when targeting windows. elsewhere each
  script is a no-op, even if `BRAHMA_REQUIRE_WINRES` is set, and the web build
  doesn't touch that crate. both set ProductName to brahma, CompanyName to
  learning machines LLC and the same copyright line. the GUI description is
  "brahma image editor" (`brahma.exe`). the CLI description is
  "brahma command-line interface", with OriginalFilename `brahma-cli.exe`
  and InternalName `brahma-cli`. version strings are the cargo package version,
  so a pre-release tag is kept; the numeric file and product versions are
  winresource's `major.minor.patch.0`. a missing resource compiler warns, so an
  optional cross-compile still links. `package.ps1` sets `BRAHMA_REQUIRE_WINRES=1`.
  the CLI build script then returns an error naming `rc.exe` or `windres` and does
  not panic.
- release builds use the GUI subsystem, so start menu launches don't open a console window.
  the CLI stays console subsystem 3 so its output reaches the terminal.
- `brahma.wxs` (wix v5) is a per-machine install into program files with an advertised
  start menu shortcut. brahma becomes the default app for `.pcraft` and is listed under
  "open with" for PSD/PSB and image files. it also registers app paths (win+r `brahma`).
  the MSI version is the numeric `X.Y.Z`, because MSI has no pre-release field. same-version
  upgrades are allowed so that release candidates replace each other.
- double-clicking the MSI opens a setup wizard (wixui_installdir without the licence page):
  welcome, install folder (remembered for upgrades in `HKLM\Software\brahma\InstallDir`),
  ready, a progress page, and a finish page that says where to find the app, with a ticked "launch
  brahma" box. the pages name the full version (`brahma.en-us.wxl`). `package.ps1` draws
  the wizard's banner and side bitmaps from the app icon, so no wix stock art ships. `msiexec /qn`
  still installs silently. building needs the wix ui and util extensions:
  `wix extension add -g WixToolset.UI.wixext/5.0.2 WixToolset.Util.wixext/5.0.2`.
- shortcut icon identifiers keep the executable's `.exe` extension: MSI uses the identifier
  as the cached icon filename, and an extensionless filename can render as a blank document
  icon. `packaging/windows/check-icons.ps1` checks the references and extensions in ci;
  `package.ps1` also validates the built MSI with ICE50 before signing it.
- **portable zip:** it ships `packaging/windows/portable.txt` beside `brahma.exe`. that
  marker (or a `brahma.portable` file) switches on portable mode: preferences, presets,
  recovery autosaves and the GPU startup marker go to `BrahmaData\` next to the exe instead
  of `%APPDATA%\Brahma`. if that folder isn't writable the app warns and uses `%APPDATA%`.
  the MSI has no marker. the logic is in `apps/brahma/src/app_dirs.rs` and works the same
  on macos and linux.
- **signing:** `packaging/windows/sign.ps1` signs both `.exe` files and then the `.msi` with
  `signtool`, using SHA-256 and an RFC 3161 timestamp. it uses whichever material is present:
  1. a `.pfx` file (`WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`), timestamped by
     DigiCert, or
  2. azure trusted signing (`AZURE_*`): the script downloads the
     `Microsoft.Trusted.Signing.Client` dlib and timestamps with Microsoft's server.

  the script is the single place to change when the windows signing setup changes.

locally on windows: `dotnet tool install -g wix --version 5.0.2`,
`wix extension add -g WixToolset.UI.wixext/5.0.2 WixToolset.Util.wixext/5.0.2`, then
`pwsh packaging/windows/package.ps1 -Arch x64`.

### linux

`packaging/linux/package.sh` stages one FHS tree and makes every format from it. the tree
holds both binaries, `com.houseofasher.brahma.desktop`, hicolor icons from 16 px to 512 px
plus a scalable SVG, AppStream metainfo, and a shared-mime-info file for `.pcraft`, `.psb` and
`.qoi`.

why these formats:

- **AppImage** runs on any distribution without installing anything. the
  download-and-go option and the fallback for distros the packages don't cover. it's built
  with the maintained `AppImage/appimagetool`, whose static runtime doesn't need libfuse2.
  the packaging script verifies pinned AppImageTool 1.9.1 and type2 runtime 20251108 binaries.
  for an offline build, pre-seed `$CARGO_TARGET_DIR/appimagetool-1.9.1-$ARCH.AppImage` and
  `$CARGO_TARGET_DIR/type2-runtime-20251108-$ARCH` with the matching official release assets;
  the script checks both SHA-256 digests and makes the cached AppImageTool executable after verification;
  the runtime is only read as data.
  an `APPIMAGETOOL` override must point to that same verified AppImageTool binary and already be
  executable (`chmod +x` it before packaging).
  each AppImage embeds update information (`gh-releases-zsync|…|latest|…`, #349) and ships
  with a `.zsync` beside it, so AppImageUpdate, AppImageLauncher and similar tools can find
  the next release and download only the blocks that changed.
  its `AppRun` (`packaging/linux/AppRun`) also integrates with the desktop on launch: on
  wayland the dock icon comes from the compositor resolving the window's app_id against an
  *installed* `.desktop` file, which an AppImage alone never provides — without this every
  AppImage run shows the generic wayland logo (#593). best effort: it installs
  `~/.local/share/applications/com.houseofasher.brahma.desktop` with `Exec` rewritten to
  the AppImage's path (`TryExec` dropped, the binary isn't on the host PATH) and the
  hicolor icons beside it, rewriting only when the content changed, and any failure leaves
  the launch untouched. `BRAHMA_NO_DESKTOP_INTEGRATION=1` opts out; to uninstall, `rm
  ~/.local/share/applications/com.houseofasher.brahma.desktop` and `find
  ~/.local/share/icons/hicolor -name 'com.houseofasher.brahma*' -delete`. the script is
  covered by `packaging/linux/apprun-test.sh` in the packaging-lint workflow.
- **.deb** covers debian, ubuntu, mint, pop!_os and elementary. **.rpm** covers fedora, RHEL
  and its clones, and opensuse. both integrate with the menu, MIME and icon caches (the
  `postinst.sh` hook) and uninstall cleanly. both are built by
  nfpm: one maintained, dependency-free tool and one config
  (`nfpm.yaml`). it doesn't need cargo-deb and cargo-generate-rpm metadata in our crates, and
  it can build both formats on ubuntu.
- **.tar.gz** is for people who manage their own `/opt` or `~/.local`.
- **flatpak bundle** (`.flatpak`) is a single file that installs into flatpak, sandboxed and
  updatable by installing a newer bundle, for people who prefer flatpak to AppImage.
  `packaging/linux/flatpak-bundle.sh` repackages the job's `.tar.gz` with
  `packaging/linux/flatpak/com.houseofasher.brahma.bundle.yml` on the freedesktop 26.08
  runtime. it does no rust build and needs no network inside `flatpak-builder`, so the bundle
  holds the same binaries as the other formats. a separate `flatpak` job per architecture
  (`ubuntu-24.04` and `ubuntu-24.04-arm`, for flatpak-builder 1.4) downloads the linux job's
  artifact, runs the script, installs the bundle and runs `brahma-cli --version` inside the
  sandbox as a smoke test. the bundle names flathub as its runtime repo, so users install
  it with:

  ```sh
  flatpak install --user brahma-<v>-linux-x86_64.flatpak   # pulls org.freedesktop.Platform//26.08 from Flathub if missing
  flatpak run com.houseofasher.brahma
  ```

  sandbox permissions (justified in the manifest): wayland with X11 fallback, IPC (X11
  shared memory), `dri` for the GPU, and read/write access to pictures and documents. every
  other file goes through the file-chooser and document portals. there's no network, so the
  `--control` server is only reachable from inside the sandbox until the user runs
  `flatpak override --user --share=network com.houseofasher.brahma`. host fonts are read from
  `/run/host/fonts` and `/run/host/user-fonts`. locally (on linux, with `flatpak` and
  `flatpak-builder`): `packaging/linux/package.sh --formats tar && packaging/linux/flatpak-bundle.sh`.
- **flathub**: `packaging/linux/flatpak/com.houseofasher.brahma.yml` builds from source and
  is ready for a flathub submission (it keeps the same runtime and `finish-args` as the bundle
  manifest, which packaging-lint checks). ci doesn't build it. a real build needs vendored crate
  sources (`cargo-sources.json` from `flatpak-cargo-generator.py`) and adds 20+ minutes per
  architecture. build it by hand with the commands in the manifest's header.

all linux binaries are built on ubuntu 22.04 and need **glibc ≥ 2.35**: ubuntu 22.04+,
debian 12+, fedora 36+, RHEL 10, opensuse tumbleweed. they link only glibc and libgcc_s. X11,
wayland, xkbcommon, vulkan and EGL are loaded at runtime from the system, which is also where
the GPU driver has to come from. that's why the .deb and .rpm declare them as dependencies
(the full list, and why each is there, is in `packaging/linux/nfpm.yaml`) and the AppImage
doesn't bundle them. the flatpak gets them from the freedesktop runtime. because the AppImage
and the tarball can't declare dependencies, `brahma` checks for the libraries its session
(X11 or wayland) needs before it opens a window (`apps/brahma/src/linux_libs.rs`) and, if
one is missing, prints the package to install and exits with status 1 instead of crashing.
`BRAHMA_SKIP_LIB_CHECK=1` skips the check.

locally (on linux): install nfpm, then
`packaging/linux/package.sh` (or `--formats "deb tar"`).

### freebsd

GitHub has no freebsd runners, so the `freebsd` job runs `packaging/freebsd/package.sh` in a
freebsd 14.3 vm (`vmactions/freebsd-vm`, pinned by commit), with the same packages as the
freebsd ci workflow (`.github/workflows/freebsd.yml`) plus `bash`. the script builds both
binaries with `CARGO_BUILD_JOBS=4` (more runs the 12 gb vm out of memory) and writes
`brahma-<v>-freebsd-x86_64.tar.gz`. freebsd's `uname -m` says `amd64`; the file name uses
`x86_64` like the other artifacts. the tarball is a `/usr/local`-style tree: `bin/brahma`,
`bin/brahma-cli`, and under `share/` the same desktop entry, MIME type, AppStream metainfo
and hicolor icons as linux, plus the licences, `NOTICE` and `ATTRIBUTION.md` in
`share/doc/brahma/`. users install it with:

```sh
pkg install libxkbcommon wayland libX11 libXcursor libXrandr libXi libxcb mesa-libs vulkan-loader gtk3 fontconfig freetype2 alsa-lib
tar -xzf brahma-<v>-freebsd-x86_64.tar.gz --strip-components 1 -C /usr/local
```

the job signs nothing, so it runs outside the `release` environment and gets no secrets. the
version, build date and commit reach the vm through the action's `envs:` (by name, never
templated into the script), and the build happens in `/var/tmp` so only `dist/` is copied back
out of the vm. the binaries aren't signed: freebsd has no code-signing scheme for loose
binaries, so check the tarball against `SHA256SUMS.txt`.

the freebsd ci workflow runs the same script after its build and test (on pushes to `main`, and
on prs that touch `packaging/freebsd/`, `freebsd.yml` or `release.yml`), so the release job
isn't the first place it runs. locally, on any os, `packaging/freebsd/package.sh --dry-run` stages the tree from stub binaries
and lists the tarball, which checks the layout without a freebsd machine.

### web

`packaging/web/package.sh` runs `trunk build --release` (see `apps/brahma-web/Trunk.toml`)
and zips `dist/web` together with sample `_headers` and `.htaccess` files and the hosting guide.
the site only uses relative urls, so it works under any path and in an iframe. the wasm builds
with the size-optimized `wasm-release` cargo profile (set in `apps/brahma-web/index.html`),
and the script fails if any `.wasm` exceeds 24 mib, below cloudflare's 25 mib per-file limit.
[`packaging/web/README.md`](../packaging/web/README.md) covers MIME types, compression,
caching, the iframe snippet and the `?webgl` / `?cpu` flags.

## secrets

all secrets live in the repository's **`release` environment** (*settings → environments →
release*). restrict its deployment branches to `release`. every job in `release.yml` that
signs or publishes declares `environment: release` (the flatpak and freebsd jobs sign nothing
and don't), so only pushes to that branch, or manual runs on it, can read the secrets. each secret is optional. if one is missing, that platform's artifacts are unsigned and
the run shows a `::warning::`.

| secret | used for |
|---|---|
| `APPLE_CERTIFICATE` | base64 `.p12` with "developer id application: learning machines LLC (DJ6XS33FX8)" |
| `APPLE_CERTIFICATE_PASSWORD` | password for that `.p12` |
| `KEYCHAIN_PASSWORD` | password for the temporary ci keychain (random if unset) |
| `APPLE_ID` | apple id used by `notarytool` |
| `APPLE_PASSWORD` | app-specific password for that apple id |
| `APPLE_TEAM_ID` | `DJ6XS33FX8` |
| `WINDOWS_CERTIFICATE` | base64 `.pfx` code-signing certificate |
| `WINDOWS_CERTIFICATE_PASSWORD` | password for that `.pfx` |
| `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET` | service principal for azure trusted signing (an alternative to the `.pfx`) |
| `AZURE_SIGNING_ENDPOINT` | e.g. `https://eus.codesigning.azure.net` |
| `AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE` | trusted signing account and certificate profile names |

`GITHUB_TOKEN` creates the release. only the final job gets `contents: write`.

on macos, `packaging/macos/import-cert.sh` decodes the `.p12` into a temporary keychain
(`security create-keychain`, then `security import`, then `set-key-partition-list` so codesign
can use the key without a prompt). it exports the identity's SHA-1 as `MACOS_SIGN_IDENTITY`.
the keychain is deleted at the end of the job.

## icons

`assets/app-icon/brahma.svg` is the canonical icon: the owner's #houseofasher drawing of a
nine-tailed kitsune, vectorised (MIT or apache-2.0, see `LICENSE.txt` there; palette and
geometry in `README.md`). `packaging/icons.sh` regenerates the 1024 px PNG, the `.icns`, the `.ico` (packed by
`cargo xtask ico`) and the hicolor pngs from it. it needs `resvg`, plus `iconutil` on macos.
the outputs are committed, so packaging never needs those tools.

## checks

`.github/workflows/packaging-lint.yml` runs in seconds on any change to `packaging/`, the
workflows or the icons. it runs actionlint, shellcheck, a PowerShell parse, xmllint,
`desktop-file-validate`, `appstreamcli validate`, and a YAML check that the two flatpak
manifests agree on the runtime and permissions.
