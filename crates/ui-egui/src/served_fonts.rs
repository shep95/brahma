//! Fonts a host serves next to the app, for the platform shell that fetches them (the web build;
//! see `brahma_text::served`): reading the manifest, the families to fetch, and installing
//! what arrived, which registers the files and re-renders the type layers drawn without them.

use std::collections::HashSet;
use std::sync::{Mutex, PoisonError};

use brahma_doc::{DocId, LayerContent, LayerId, TextLayer};
pub use brahma_text::served::{ServedFont, add_families, add_fonts, files_for, google_fonts, parse_manifest, request_for_languages, take_requests};

use crate::BrahmaApp;

/// Layers whose document was busy with a background job when their font arrived.
static PENDING: Mutex<Vec<(DocId, LayerId)>> = Mutex::new(Vec::new());

/// Families fetched or being fetched (each once per run), and files that arrived since the
/// last frame as `(family, bytes)`.
static FETCHED: Mutex<Vec<String>> = Mutex::new(Vec::new());
static ARRIVED: Mutex<Vec<(String, Vec<u8>)>> = Mutex::new(Vec::new());
static CATALOG_ADDED: Mutex<bool> = Mutex::new(false);

/// Every frame, for shells with a fetcher (`Services::fetch_url`): on the first frame lists the
/// Google Fonts catalogue (about 1,900 families under free licences, fetched from a pinned
/// commit when first used); then fetches the families asked for (picked in a font menu, or
/// needed by a layout) and installs what arrived.
pub fn serve(app: &mut BrahmaApp, ctx: &egui::Context) {
    if app.services.fetch_url.is_none() {
        return;
    }
    {
        let mut added = CATALOG_ADDED.lock().unwrap_or_else(PoisonError::into_inner);
        if !*added {
            *added = true;
            add_fonts(&google_fonts());
        }
    }
    let requested = take_requests();
    if !requested.is_empty() {
        let mut fetched = FETCHED.lock().unwrap_or_else(PoisonError::into_inner);
        let mut seen: HashSet<String> = fetched.iter().cloned().collect();
        for family in requested {
            if !seen.insert(family.clone()) {
                continue;
            }
            fetched.push(family.clone());
            for (file, integrity) in files_for(&family) {
                let Some(fetch) = app.services.fetch_url.as_ref() else { break };
                let family = family.clone();
                let repaint = ctx.clone();
                let shown = file.clone();
                fetch(
                    file,
                    integrity,
                    Box::new(move |outcome| {
                        match outcome {
                            Ok(bytes) => ARRIVED.lock().unwrap_or_else(PoisonError::into_inner).push((family, bytes)),
                            Err(e) => log::error!("couldn't fetch font {shown}: {e}"),
                        }
                        repaint.request_repaint();
                    }),
                );
            }
        }
    }
    let arrived = std::mem::take(&mut *ARRIVED.lock().unwrap_or_else(PoisonError::into_inner));
    install(app, arrived);
}

/// Registers delivered font files (`(family, bytes)`, the family as the manifest lists it) and
/// re-renders the type layers that use those families, or whose text falls back to them (a script
/// fallback, e.g. Arabic set in Inter), and show our own layout. A PSD layer still
/// showing Photoshop's pixels (not edited yet) keeps them, as it does when its font is installed
/// from the start. Call it every frame: it also retries the layers of a document that was busy.
pub fn install(app: &mut BrahmaApp, delivered: Vec<(String, Vec<u8>)>) {
    let mut layers = std::mem::take(&mut *PENDING.lock().unwrap_or_else(PoisonError::into_inner));
    if !delivered.is_empty() {
        let families: Vec<String> = delivered.iter().map(|(f, _)| f.clone()).collect();
        // Decided before the fonts are in: "shows our own layout" compares the layer's pixels
        // with what the engine draws now, that is with the fallback font.
        for st in app.session.documents() {
            for (_, _, l) in st.doc.walk() {
                if let LayerContent::Text(t) = &l.content
                    && redraws_for(t, &families)
                    && crate::type_tool::shows_own_layout(&st.doc, t)
                    && !layers.contains(&(st.doc.id, l.id))
                {
                    layers.push((st.doc.id, l.id));
                }
            }
        }
        let mut eng = brahma_text::shared().lock().unwrap_or_else(PoisonError::into_inner);
        for (family, bytes) in delivered {
            crate::adaptive::font_arrived(&family, &bytes);
            let added = eng.fonts.register_font_data(bytes);
            if !added.contains(&family) {
                log::warn!("a served font file listed as {family:?} registers {added:?}; the family in fonts/manifest.txt must be the font's own");
            }
        }
    }
    if !layers.is_empty() {
        let busy = app.session.refresh_type_layers(&layers);
        PENDING.lock().unwrap_or_else(PoisonError::into_inner).extend(busy);
    }
}

/// Should the layer be redrawn when `families` arrive: it uses one of them, or its text is in a
/// script one of them is the served fallback for?
fn redraws_for(t: &TextLayer, families: &[String]) -> bool {
    uses_family(t, families) || families.iter().any(|f| brahma_text::served::falls_back_to(&t.text, f))
}

/// Does the layer use one of `families`, by name or through the PostScript name a PSD gave?
fn uses_family(t: &TextLayer, families: &[String]) -> bool {
    t.char_runs().iter().any(|r| {
        families.contains(&r.style.font_family)
            || r.style.postscript_name.as_deref().is_some_and(|ps| families.contains(&brahma_text::fonts::guess_from_postscript(ps).family))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// A shell with a fetcher lists the catalogue on its first frame; picking a family fetches
    /// its files once and installs the bytes that come back.
    #[test]
    fn a_requested_catalogue_family_is_fetched_once_and_installed() {
        let fetched: Arc<Mutex<Vec<String>>> = Arc::default();
        let log = fetched.clone();
        let services = crate::Services {
            fetch_url: Some(Box::new(move |url, _integrity, reply| {
                log.lock().unwrap().push(url.clone());
                // Any bytes: the engine rejects what isn't a font without a crash.
                reply(Ok(b"not a font".to_vec()));
            })),
            ..Default::default()
        };
        let mut app = BrahmaApp::new(brahma_engine::Session::new(), services);
        let ctx = egui::Context::default();
        serve(&mut app, &ctx);
        assert!(brahma_text::served::families().iter().any(|f| f == "Open Sans"), "the catalogue is listed");
        brahma_text::served::request("Open Sans");
        brahma_text::served::request("Open Sans");
        serve(&mut app, &ctx);
        serve(&mut app, &ctx);
        let urls = fetched.lock().unwrap().clone();
        assert_eq!(urls.len(), 2, "{urls:?}");
        assert!(urls.iter().all(|u| u.starts_with("https://raw.githubusercontent.com/google/fonts/") && u.contains("/ofl/opensans/")), "{urls:?}");
        brahma_text::served::request("Open Sans");
        serve(&mut app, &ctx);
        assert_eq!(fetched.lock().unwrap().len(), 2, "fetched once per run");
    }

    #[test]
    fn a_layer_uses_a_family_by_name_or_postscript_name() {
        let t = TextLayer { text: "Hi".into(), font_family: "Open Sans".into(), ..Default::default() };
        assert!(uses_family(&t, &["Open Sans".into()]));
        assert!(!uses_family(&t, &["Lato".into()]));
        let mut psd = TextLayer { text: "Hi".into(), font_family: "Inter".into(), ..Default::default() };
        psd.runs = vec![brahma_doc::text::TextRun {
            len: 2,
            style: brahma_doc::text::CharStyle { font_family: "Inter".into(), postscript_name: Some("Lato-Bold".into()), ..Default::default() },
        }];
        assert!(uses_family(&psd, &["Lato".into()]));
    }

    #[test]
    fn a_layer_in_a_script_redraws_when_that_scripts_fallback_arrives() {
        // Thai: other tests use other scripts (the served catalog is process-wide).
        let thai = "Served Test Thai";
        brahma_text::served::add_fonts(&parse_manifest(&format!("{thai} | Regular | fonts/t/T.ttf | Thai\n")).0);
        let layer = |text: &str| TextLayer { text: text.into(), font_family: "Inter".into(), ..Default::default() };
        assert!(redraws_for(&layer("สวัสดี"), &[thai.into()]), "Thai text set in Inter falls back to it");
        assert!(!redraws_for(&layer("hello"), &[thai.into()]));
        assert!(redraws_for(&layer("hello"), &["Inter".into()]), "and a layer still redraws for its own family");
    }
}
