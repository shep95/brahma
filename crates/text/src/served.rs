//! Fonts a host serves next to the app, fetched on demand (the web build: no system fonts, and
//! the wasm has no room for more embedded ones, see `build.rs`).
//!
//! The host lists them in `fonts/manifest.txt` next to `index.html`, in craft-fonts' manifest
//! format (`family | style | file | …`, the file relative to the site root), so a craft-fonts
//! checkout's `fonts/` folder works as it is and any other font needs one line. This module is
//! the part that doesn't depend on the platform: reading the manifest, the catalog of families
//! the host can serve, and the queue of families to fetch (picked in a font menu, or needed by a
//! layout). The web shell does the fetching and registers what arrives with
//! [`crate::FontDb::register_font_data`].
//!
//! Script fallback: the manifest's `scripts` field says which scripts a font is for. The first
//! served family listed for a script the bundled fonts don't cover (Arabic, Hebrew, Devanagari,
//! Thai, Khmer, Lao, Burmese, Japanese, Chinese, Korean) is that script's fallback: text in that script asks for it
//! ([`request_for_text`], from layout), the font database puts it in the fallback stack once it
//! arrives, and the web shell asks up front for the scripts of the browser's languages
//! ([`request_for_languages`]). The manifest's SHA-256 becomes the fetch's Subresource Integrity
//! ([`ServedFont::integrity`]).

use std::sync::{Mutex, OnceLock, PoisonError};

/// One font file listed in a served font manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServedFont {
    /// The family the file registers as (its typographic family name, `name` ID 16, else ID 1).
    pub family: String,
    /// Path relative to the site root, e.g. `fonts/open-sans/OpenSans-Regular.ttf`, or an
    /// absolute `https://` URL (the Google Fonts catalogue, [`google_fonts`]).
    pub file: String,
    /// ISO 15924 scripts the manifest lists for the file (`Arab`, `Jpan`, …); empty when absent.
    pub scripts: Vec<String>,
    /// Subresource Integrity value (`sha256-<base64>`) from the manifest's SHA-256, when it has a
    /// valid one: the browser rejects any other bytes.
    pub integrity: Option<String>,
}

/// Reads a manifest in craft-fonts' format: one font file per line, fields separated by ` | `
/// (`family | style | file | scripts | licence | licence file | sha256 | source`); `#` comments
/// and blank lines are ignored. Only family and file are needed here, so a line needs at least
/// `family | style | file`. Returns the fonts and, for each line that was skipped, why.
pub fn parse_manifest(text: &str) -> (Vec<ServedFont>, Vec<String>) {
    let mut fonts = Vec::new();
    let mut skipped = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let f: Vec<&str> = line.split(" | ").map(str::trim).collect();
        match f.as_slice() {
            [family, _style, file, rest @ ..] if !family.is_empty() && is_relative_path(file) => {
                let scripts = rest.first().map(|s| s.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect()).unwrap_or_default();
                let integrity = rest.get(3).and_then(|sha| sri(sha));
                fonts.push(ServedFont { family: (*family).to_string(), file: (*file).to_string(), scripts, integrity });
            }
            [_, _, file, ..] if !is_relative_path(file) => skipped.push(format!("{line}: the file must be a relative path inside the site")),
            _ => skipped.push(format!("{line}: expected `family | style | file | …`")),
        }
    }
    (fonts, skipped)
}

/// A relative path that stays inside the site: no scheme, no leading `/`, no `..` segment, no
/// query or fragment.
fn is_relative_path(p: &str) -> bool {
    !p.is_empty() && !p.starts_with('/') && !p.contains(['\\', '?', '#', ':']) && p.split('/').all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

/// The scripts a served font can be the fallback for: those the bundled fonts (Inter, which covers
/// Latin, Greek and Cyrillic) lack, as ISO 15924 codes.
const FALLBACK_SCRIPTS: &[&str] = &["Arab", "Hebr", "Deva", "Thai", "Khmr", "Laoo", "Mymr", "Jpan", "Hans", "Hant", "Kore"];

/// The script fallbacks in `fonts`: for each script in [`FALLBACK_SCRIPTS`], the first family in
/// manifest order that lists it, as `(script, family)` in order of first appearance.
pub fn script_fallbacks(fonts: &[ServedFont]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for f in fonts {
        for s in &f.scripts {
            if FALLBACK_SCRIPTS.contains(&s.as_str()) && !out.iter().any(|(have, _)| have == s) {
                out.push((s.clone(), f.family.clone()));
            }
        }
    }
    out
}

/// The fallback script of `c` (one of [`FALLBACK_SCRIPTS`]), or `None` for characters the bundled
/// fonts cover. Han follows `order` (the UI locale's CJK order), as the installed CJK fonts do.
pub fn script_of(c: char, order: &[crate::cjk::CjkScript; 4]) -> Option<&'static str> {
    use crate::cjk::CjkScript;
    if let Some(k) = crate::cjk::classify(c) {
        return Some(match k.preferred(order) {
            CjkScript::Japanese => "Jpan",
            CjkScript::SimplifiedChinese => "Hans",
            CjkScript::TraditionalChinese => "Hant",
            CjkScript::Korean => "Kore",
        });
    }
    Some(match c as u32 {
        0x0590..=0x05FF | 0xFB1D..=0xFB4F => "Hebr",
        0x0600..=0x06FF | 0x0750..=0x077F | 0x0870..=0x08FF | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF | 0x1EE00..=0x1EEFF => "Arab",
        0x0900..=0x097F | 0xA8E0..=0xA8FF => "Deva",
        0x0E00..=0x0E7F => "Thai",
        0x1780..=0x17FF | 0x19E0..=0x19FF => "Khmr",
        0x0E80..=0x0EFF => "Laoo",
        0x1000..=0x109F | 0xA9E0..=0xA9FF | 0xAA60..=0xAA7F => "Mymr",
        _ => return None,
    })
}

/// The fallback script a browser language (`ar-EG`, `zh-Hant-TW`, …) is written in, if any.
pub fn language_script(tag: &str) -> Option<&'static str> {
    let tag = tag.trim().to_ascii_lowercase().replace('_', "-");
    let mut parts = tag.split('-');
    let lang = parts.next()?;
    let rest: Vec<&str> = parts.collect();
    Some(match lang {
        "ar" | "fa" | "ur" | "ps" | "ckb" | "sd" | "ug" => "Arab",
        "he" | "iw" | "yi" => "Hebr",
        "hi" | "mr" | "ne" | "sa" => "Deva",
        "th" => "Thai",
        "km" => "Khmr",
        "lo" => "Laoo",
        "my" => "Mymr",
        "ja" => "Jpan",
        "ko" => "Kore",
        "zh" if rest.iter().any(|r| matches!(*r, "hant" | "tw" | "hk" | "mo")) => "Hant",
        "zh" => "Hans",
        _ => return None,
    })
}

/// The Subresource Integrity value (`sha256-<base64>`) for a 64-digit hex SHA-256.
fn sri(hex: &str) -> Option<String> {
    if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..32).map(|i| hex.get(2 * i..2 * i + 2).and_then(|h| u8::from_str_radix(h, 16).ok())).collect();
    Some(format!("sha256-{}", base64(&bytes?)))
}

/// Standard base64 (RFC 4648, with padding).
fn base64(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = |i: usize| u32::from(chunk.get(i).copied().unwrap_or(0));
        let n = (b(0) << 16) | (b(1) << 8) | b(2);
        for k in 0..4 {
            let six = ((n >> (18 - 6 * k)) & 63) as usize;
            s.push(if k <= chunk.len() { TABLE.get(six).map_or('=', |&c| char::from(c)) } else { '=' });
        }
    }
    s
}

/// A file to fetch and its Subresource Integrity value, when known.
pub type FileRef = (String, Option<String>);

struct State {
    /// Families the host can serve, installed or not.
    families: Vec<String>,
    /// The files of each family.
    files: Vec<(String, Vec<FileRef>)>,
    /// Script fallbacks, `(script, family)`: the first one per script is used.
    fallbacks: Vec<(String, String)>,
    /// Families to fetch, not yet taken by [`take_requests`].
    requested: Vec<String>,
    /// Bumped when `families` changes, so font menus refresh.
    generation: u64,
}

static STATE: Mutex<State> = Mutex::new(State { families: Vec::new(), files: Vec::new(), fallbacks: Vec::new(), requested: Vec::new(), generation: 0 });

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Adds families the host can serve (the web shell, after reading the manifest).
pub fn add_families(families: impl IntoIterator<Item = String>) {
    let mut s = state();
    let before = s.families.len();
    for f in families {
        if !s.families.contains(&f) {
            s.families.push(f);
        }
    }
    if s.families.len() != before {
        s.generation = s.generation.wrapping_add(1);
    }
}

/// Adds the fonts of a manifest: their families (as [`add_families`]), their files (the first
/// listing of a family wins, so a host's own manifest beats the catalogue when it comes first)
/// and their script fallbacks.
pub fn add_fonts(fonts: &[ServedFont]) {
    add_families(fonts.iter().map(|f| f.family.clone()));
    let mut s = state();
    for f in fonts {
        let idx = match s.files.iter().position(|(fam, _)| *fam == f.family) {
            Some(i) => i,
            None => {
                s.files.push((f.family.clone(), Vec::new()));
                s.files.len() - 1
            }
        };
        if let Some((_, entry)) = s.files.get_mut(idx)
            && !entry.iter().any(|(file, _)| *file == f.file)
        {
            entry.push((f.file.clone(), f.integrity.clone()));
        }
    }
    for pair in script_fallbacks(fonts) {
        if !s.fallbacks.contains(&pair) {
            s.fallbacks.push(pair);
        }
    }
}

/// The files to fetch for a served family (empty for one nobody listed).
pub fn files_for(family: &str) -> Vec<FileRef> {
    state().files.iter().find(|(fam, _)| fam == family).map(|(_, v)| v.clone()).unwrap_or_default()
}

/// The commit of github.com/google/fonts the catalogue was distilled from; every file is fetched
/// from it, so a family never changes under a document.
pub const GOOGLE_FONTS_COMMIT: &str = "47d54ca1ebb14fd864695f1d39dc734f39017c33";

/// Where a catalogue file lives: the pinned commit on GitHub's raw host (CORS-enabled).
pub const GOOGLE_FONTS_BASE: &str = "https://raw.githubusercontent.com/google/fonts";

/// One family of the Google Fonts catalogue (`data/google_fonts.tsv`, distilled by
/// `scripts/google_fonts_catalog.py` from the repository's `METADATA.pb` files; every family is
/// under the OFL, Apache or UFL licence, and no font bytes ship with brahma).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogFont {
    pub family: &'static str,
    /// `sans-serif`, `serif`, `display`, `handwriting` or `monospace`.
    pub category: &'static str,
    /// Directory in the repository, e.g. `ofl/opensans`.
    pub dir: &'static str,
    /// File names to fetch: the variable fonts, or regular, italic, bold and bold italic.
    pub files: Vec<&'static str>,
    /// Fallback scripts (ISO 15924) the family's subsets cover.
    pub scripts: Vec<&'static str>,
}

const GOOGLE_FONTS_TSV: &str = include_str!("../data/google_fonts.tsv");

/// The catalogue, parsed once.
pub fn google_catalog() -> &'static [CatalogFont] {
    static CATALOG: OnceLock<Vec<CatalogFont>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        GOOGLE_FONTS_TSV
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .filter_map(|l| {
                let mut f = l.split('\t');
                let (family, category, dir, files) = (f.next()?.trim(), f.next()?.trim(), f.next()?.trim(), f.next()?.trim());
                // `|` separates files and scripts: variable font names carry commas (`[wdth,wght]`).
                let scripts = f.next().unwrap_or("").split('|').map(str::trim).filter(|s| !s.is_empty()).collect();
                let files: Vec<&'static str> = files.split('|').map(str::trim).filter(|s| !s.is_empty()).collect();
                if family.is_empty() || dir.is_empty() || files.is_empty() || !is_relative_path(dir) || files.iter().any(|x| !is_relative_path(x)) {
                    return None;
                }
                Some(CatalogFont { family, category, dir, files, scripts })
            })
            .collect()
    })
}

/// The catalogue as served fonts, one per file, ready for [`add_fonts`]. The URL points at the
/// pinned commit; the shell fetches it the first time the family is picked or needed.
pub fn google_fonts() -> Vec<ServedFont> {
    google_catalog()
        .iter()
        .flat_map(|c| {
            c.files.iter().map(move |file| ServedFont {
                family: c.family.to_string(),
                // Variable font names carry `[axes]`: percent-encoded, so every HTTP client and
                // the browser's URL parser agree on the path.
                file: format!("{GOOGLE_FONTS_BASE}/{GOOGLE_FONTS_COMMIT}/{}/{}", c.dir, file.replace('[', "%5B").replace(']', "%5D")),
                scripts: c.scripts.iter().map(|s| (*s).to_string()).collect(),
                integrity: None,
            })
        })
        .collect()
}

/// The catalogue's category of a family (`None` for fonts from elsewhere), for font menus that
/// group or filter by it.
pub fn google_category(family: &str) -> Option<&'static str> {
    google_catalog().iter().find(|c| c.family == family).map(|c| c.category)
}

/// The served families that are some script's fallback (the font database adds the ones it has
/// to its fallback stack).
pub fn script_fallback_families() -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for (_, f) in &state().fallbacks {
        if !v.contains(f) {
            v.push(f.clone());
        }
    }
    v
}

/// The fallback family for `script`, if the host serves one.
fn fallback_for(fallbacks: &[(String, String)], script: &str) -> Option<String> {
    fallbacks.iter().find(|(s, _)| s == script).map(|(_, f)| f.clone())
}

/// Asks for the served fallback families `text` needs: one per script in it that the bundled
/// fonts lack (called by layout; cheap when the host serves no script fallbacks).
pub fn request_for_text(text: &str) {
    let fallbacks = state().fallbacks.clone();
    if fallbacks.is_empty() {
        return;
    }
    let order = crate::cjk::ui_script_order();
    let mut asked: Vec<&'static str> = Vec::new();
    for c in text.chars() {
        if let Some(script) = script_of(c, &order)
            && !asked.contains(&script)
        {
            asked.push(script);
            if let Some(family) = fallback_for(&fallbacks, script) {
                request(&family);
            }
        }
    }
}

/// Does `text` contain characters whose served fallback is `family`? (Type layers drawn before it
/// arrived are re-rendered.)
pub fn falls_back_to(text: &str, family: &str) -> bool {
    let fallbacks = state().fallbacks.clone();
    if !fallbacks.iter().any(|(_, f)| f == family) {
        return false;
    }
    let order = crate::cjk::ui_script_order();
    text.chars().any(|c| script_of(c, &order).and_then(|s| fallback_for(&fallbacks, s)).is_some_and(|f| f == family))
}

/// Asks up front for the fallback families of the scripts `languages` (the browser's, e.g.
/// `navigator.languages`) are written in, so text in them never waits for its font.
pub fn request_for_languages<'a>(languages: impl IntoIterator<Item = &'a str>) {
    let fallbacks = state().fallbacks.clone();
    for script in languages.into_iter().filter_map(language_script) {
        if let Some(family) = fallback_for(&fallbacks, script) {
            request(&family);
        }
    }
}

/// Families the host can serve, installed or not (for font menus).
pub fn families() -> Vec<String> {
    state().families.clone()
}

/// Changes whenever [`families`] does.
pub fn generation() -> u64 {
    state().generation
}

/// Asks for a family to be fetched: picked in a font menu, or needed by a layout that doesn't
/// have it. Families the host doesn't serve are ignored; asking again is harmless.
pub fn request(family: &str) {
    let mut s = state();
    if s.families.iter().any(|f| f == family) && !s.requested.iter().any(|f| f == family) {
        s.requested.push(family.to_string());
    }
}

/// The families asked for since the last call (the web shell fetches each one once).
pub fn take_requests() -> Vec<String> {
    std::mem::take(&mut state().requested)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_google_catalogue_lists_every_family_with_files_and_fallbacks_first() {
        let cat = google_catalog();
        assert!(cat.len() >= 1900, "{} families", cat.len());
        let mut seen = std::collections::HashSet::new();
        for c in cat {
            assert!(seen.insert(c.family), "duplicate {}", c.family);
            assert!(!c.files.is_empty() && c.files.len() <= 4, "{}: {:?}", c.family, c.files);
            assert!(["sans-serif", "serif", "display", "handwriting", "monospace"].contains(&c.category), "{}: {}", c.family, c.category);
            assert!(c.dir.starts_with("ofl/") || c.dir.starts_with("apache/") || c.dir.starts_with("ufl/"), "{}: {}", c.family, c.dir);
        }
        for f in ["Open Sans", "Roboto", "Lato", "Noto Sans JP", "Playfair Display", "JetBrains Mono"] {
            assert!(seen.contains(f), "missing {f}");
        }
        let fonts = google_fonts();
        assert!(fonts.len() >= cat.len());
        assert!(fonts[0].file.starts_with("https://raw.githubusercontent.com/google/fonts/47d54ca1"));
        let open_sans = fonts.iter().find(|f| f.family == "Open Sans").unwrap();
        assert!(open_sans.file.ends_with("/ofl/opensans/OpenSans%5Bwdth,wght%5D.ttf"), "{}", open_sans.file);
        // The Noto Sans families lead, so they are the script fallbacks.
        let fb = script_fallbacks(&fonts);
        for (script, family) in
            [("Arab", "Noto Sans Arabic"), ("Jpan", "Noto Sans JP"), ("Hans", "Noto Sans SC"), ("Kore", "Noto Sans KR"), ("Thai", "Noto Sans Thai")]
        {
            assert!(fb.contains(&(script.to_string(), family.to_string())), "{script}: {fb:?}");
        }
        assert_eq!(google_category("Roboto"), Some("sans-serif"));
        assert_eq!(google_category("Nope"), None);
    }

    #[test]
    fn files_of_added_fonts_are_remembered_first_listing_wins() {
        add_fonts(&[ServedFont { family: "Zz Test".into(), file: "fonts/zz/a.ttf".into(), scripts: vec![], integrity: Some("sha256-x".into()) }]);
        add_fonts(&[ServedFont { family: "Zz Test".into(), file: "https://example.org/b.ttf".into(), scripts: vec![], integrity: None }]);
        let files = files_for("Zz Test");
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], ("fonts/zz/a.ttf".to_string(), Some("sha256-x".to_string())));
        assert!(files_for("Nobody").is_empty());
    }

    #[test]
    fn reads_craft_fonts_manifest_lines() {
        let text = "# Font manifest for the #houseofasher apps.\n\
            \n\
            BIZ UDPGothic | Regular | fonts/biz-ud-pgothic/BIZUDPGothic-Regular.ttf | Jpan,Latn | OFL-1.1 | fonts/biz-ud-pgothic/OFL.txt | 258d | https://example.org\n\
            Open Sans | Regular | fonts/open-sans/OpenSans[wdth,wght].ttf\n";
        let (fonts, skipped) = parse_manifest(text);
        assert_eq!(
            fonts,
            [
                ServedFont {
                    family: "BIZ UDPGothic".into(),
                    file: "fonts/biz-ud-pgothic/BIZUDPGothic-Regular.ttf".into(),
                    scripts: vec!["Jpan".into(), "Latn".into()],
                    integrity: None, // "258d" is not a SHA-256
                },
                ServedFont { family: "Open Sans".into(), file: "fonts/open-sans/OpenSans[wdth,wght].ttf".into(), scripts: vec![], integrity: None },
            ]
        );
        assert!(skipped.is_empty(), "{skipped:?}");
    }

    #[test]
    fn skips_malformed_lines_and_paths_outside_the_site() {
        let text = "Lato | Regular\n\
            | Regular | fonts/x.ttf\n\
            A | Regular | /fonts/a.ttf\n\
            B | Regular | ../b.ttf\n\
            C | Regular | fonts/../../c.ttf\n\
            D | Regular | https://example.org/d.ttf\n\
            E | Regular | fonts//e.ttf\n\
            F | Regular | fonts\\f.ttf\n\
            G | Regular | fonts/g.ttf?v=1\n\
            Lato | Bold | fonts/lato/Lato-Bold.ttf\n";
        let (fonts, skipped) = parse_manifest(text);
        assert_eq!(fonts, [ServedFont { family: "Lato".into(), file: "fonts/lato/Lato-Bold.ttf".into(), scripts: vec![], integrity: None }]);
        assert_eq!(skipped.len(), 9, "{skipped:?}");
    }

    /// One test for the whole queue: the catalog and the queue are process-wide, and parallel
    /// tests taking each other's requests would be flaky.
    #[test]
    fn requests_are_limited_to_the_catalog_and_reach_layout() {
        let served = "Served Test Sans";
        let g = generation();
        add_families([served.to_string(), served.to_string()]);
        assert_eq!(families().iter().filter(|f| *f == served).count(), 1);
        assert_ne!(generation(), g, "the catalog changed");
        let g = generation();
        add_families([served.to_string()]);
        assert_eq!(generation(), g, "nothing new");

        request("Not Served Test Sans");
        request(served);
        request(served);
        let taken = take_requests();
        assert_eq!(taken.iter().filter(|f| *f == served).count(), 1, "{taken:?}");
        assert!(!taken.iter().any(|f| f == "Not Served Test Sans"));

        // A layout that needs a served family the database lacks asks for it (and is drawn with
        // the fallback meanwhile).
        let t = brahma_doc::TextLayer { text: "Hi".into(), font_family: served.into(), size_pt: 12.0, ..Default::default() };
        let l = crate::TextEngine::new().layout(&t, 72.0);
        assert_eq!(l.glyphs.len(), 2);
        assert!(take_requests().iter().any(|f| f == served));

        // Script fallback: text in a script the bundled fonts lack asks for the served family
        // that covers it (Hebrew here; other tests use other scripts, the state is shared).
        let hebrew = "Served Test Hebrew";
        let manifest = format!(
            "{hebrew} | Regular | fonts/h/H.ttf | Hebr,Latn | OFL-1.1 | fonts/h/OFL.txt | {} | src\n",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        add_fonts(&parse_manifest(&manifest).0);
        assert!(families().iter().any(|f| f == hebrew), "add_fonts lists the family too");
        assert!(script_fallback_families().iter().any(|f| f == hebrew));
        let _ = take_requests();
        request_for_text("Latin only, 123");
        assert!(!take_requests().iter().any(|f| f == hebrew), "Latin text needs no served fallback");
        request_for_text("Shalom שלום");
        assert!(take_requests().iter().any(|f| f == hebrew), "Hebrew text asks for the Hebrew family");
        assert!(falls_back_to("שלום", hebrew));
        assert!(!falls_back_to("hello", hebrew));
        // Startup preloading: a browser language using that script asks for it up front.
        request_for_languages(["he-IL"]);
        assert!(take_requests().iter().any(|f| f == hebrew));
        request_for_languages(["en-US", "fr"]);
        assert!(!take_requests().iter().any(|f| f == hebrew));
        // A layout of Hebrew text in the default family asks for it too.
        let t = brahma_doc::TextLayer { text: "שלום".into(), size_pt: 12.0, ..Default::default() };
        let _ = crate::TextEngine::new().layout(&t, 72.0);
        assert!(take_requests().iter().any(|f| f == hebrew), "layout asks for the script fallback");
    }

    #[test]
    fn manifest_scripts_and_sha256_become_scripts_and_integrity() {
        let sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let (fonts, _) = parse_manifest(&format!("Noto Sans Arabic | Regular | fonts/n/N.ttf | Arab | OFL-1.1 | fonts/n/OFL.txt | {sha} | src\n"));
        assert_eq!(fonts[0].scripts, ["Arab"]);
        assert_eq!(fonts[0].integrity.as_deref(), Some("sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU="));
    }

    #[test]
    fn the_first_served_family_per_script_is_its_fallback() {
        let text = "Noto Sans Arabic | Regular | fonts/a.ttf | Arab\n\
            Cairo | Regular | fonts/c.ttf | Arab,Latn\n\
            Lobster | Regular | fonts/l.ttf | Latn,Cyrl\n\
            BIZ UDPGothic | Regular | fonts/b.ttf | Jpan,Latn\n";
        let (fonts, _) = parse_manifest(text);
        assert_eq!(
            script_fallbacks(&fonts),
            [("Arab".to_string(), "Noto Sans Arabic".to_string()), ("Jpan".to_string(), "BIZ UDPGothic".to_string())],
            "Latin and Cyrillic are the bundled fonts' job"
        );
    }

    #[test]
    fn characters_map_to_the_scripts_served_fonts_cover() {
        use crate::cjk::CjkScript::*;
        let ja = [Japanese, SimplifiedChinese, TraditionalChinese, Korean];
        let zh = [SimplifiedChinese, TraditionalChinese, Japanese, Korean];
        for (c, want) in [
            ('ب', Some("Arab")),
            ('ﻻ', Some("Arab")),
            ('ש', Some("Hebr")),
            ('क', Some("Deva")),
            ('ก', Some("Thai")),
            ('ខ', Some("Khmr")),
            ('ៗ', Some("Khmr")),
            ('ລ', Some("Laoo")),
            ('မ', Some("Mymr")),
            ('ၡ', Some("Mymr")),
            ('あ', Some("Jpan")),
            ('한', Some("Kore")),
        ] {
            assert_eq!(script_of(c, &ja), want, "{c}");
        }
        for c in ['a', 'Ж', 'Ω', '1', ' ', '€'] {
            assert_eq!(script_of(c, &ja), None, "{c}: the bundled fonts cover it");
        }
        assert_eq!(script_of('漢', &ja), Some("Jpan"));
        assert_eq!(script_of('漢', &zh), Some("Hans"));
    }

    #[test]
    fn browser_languages_map_to_scripts() {
        for (tag, want) in [
            ("ar", Some("Arab")),
            ("ar-EG", Some("Arab")),
            ("fa-IR", Some("Arab")),
            ("he", Some("Hebr")),
            ("km-KH", Some("Khmr")),
            ("lo", Some("Laoo")),
            ("my-MM", Some("Mymr")),
            ("ja-JP", Some("Jpan")),
            ("zh-CN", Some("Hans")),
            ("zh-TW", Some("Hant")),
            ("zh-Hant-HK", Some("Hant")),
            ("ko", Some("Kore")),
            ("en-US", None),
            ("ru", None),
        ] {
            assert_eq!(language_script(tag), want, "{tag}");
        }
    }

    #[test]
    fn sri_rejects_non_hex_digits() {
        let ok = "0".repeat(64);
        assert!(sri(&ok).is_some());
        let plus = format!("+f{}", "0".repeat(62));
        assert!(sri(&plus).is_none());
    }
}
