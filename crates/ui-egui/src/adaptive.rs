//! Adaptive aesthetics (Preferences › Interface › Adaptive Aesthetics): with a document open,
//! the interface follows the picture. Its surfaces (the chrome, the dock, cards, fields, the
//! pasteboard) take on the image's dominant hue at a whisper of chroma, the accent turns to
//! that hue when the picture has a clear one, and the UI font becomes the body face the
//! design research recommends for the register the picture reads as (calm, trust, urgency,
//! wonder, playful: `brahma_aesthetic`), fetched from the Google Fonts catalogue the first
//! time it is needed. Close the document, or turn the preference off, and the theme returns
//! exactly as chosen.
//!
//! The analysis runs on a 96 px thumbnail, once per document revision after edits have
//! settled for a moment, so painting never pays for it per stroke.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use brahma_aesthetic::{Oklch, Register, Request};
use brahma_doc::DocId;
use egui::Color32;
use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
use egui::{FontData, FontFamily};

use crate::BrahmaApp;
use crate::theme::{self, Tokens};

/// Edits settle for this long before the picture is read again.
const SETTLE_SECS: f64 = 0.4;
/// Thumbnail side for the analysis.
const THUMB: u32 = 96;
/// Name prefix of the UI font families the adaptive font registers under.
const FONT_PREFIX: &str = "adaptive:";

/// What the picture asks of the interface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tint {
    /// The dominant hue (degrees) and chroma of the picture.
    pub hue: f32,
    pub chroma: f32,
    /// 0..1: how clearly the picture reads as its register.
    pub confidence: f32,
    pub register: Register,
}

/// Per-app state: what is applied, and the fonts fetched so far.
#[derive(Default)]
pub struct State {
    /// The document and revision the current tint was read from.
    key: Option<(DocId, u64)>,
    /// When the key last changed (seconds), for the settle delay.
    changed_at: Option<f64>,
    /// The tint in force (`None`: the plain theme).
    pub tint: Option<Tint>,
    /// The UI font family wanted for the register, if not the theme's own.
    pub font: Option<String>,
    /// The family currently registered with egui (as [`FONT_PREFIX`] + name).
    installed: Option<String>,
    /// Font bytes received, by family, so switching back never refetches.
    fonts: HashMap<String, Arc<Vec<u8>>>,
}

/// Bytes of served fonts the adaptive font may want, handed over by `served_fonts::install`.
static ARRIVED: Mutex<Vec<(String, Arc<Vec<u8>>)>> = Mutex::new(Vec::new());

/// Called by `served_fonts::install` for every delivered file.
pub fn font_arrived(family: &str, bytes: &[u8]) {
    ARRIVED.lock().unwrap_or_else(PoisonError::into_inner).push((family.to_string(), Arc::new(bytes.to_vec())));
}

/// The body face the research recommends for `register` (`None`: the theme's own font).
pub fn font_for(register: Register) -> Option<&'static str> {
    let body = brahma_aesthetic::rules::typography(&Request { register, ..Default::default() }).body;
    (body != "Inter").then_some(body)
}

/// Read the picture: its dominant colour and register.
pub fn tint_of(doc: &brahma_doc::Document) -> Option<Tint> {
    let thumb = brahma_compose::thumbnail(doc, THUMB);
    let a = brahma_aesthetic::analyze(thumb.width, thumb.height, &thumb.pixels)?;
    Some(Tint { hue: a.dominant.h, chroma: a.dominant.c, confidence: a.confidence, register: a.register })
}

/// `c` with its hue turned to `hue` and its chroma raised to a quiet `chroma`, lightness kept:
/// a grey takes the picture's cast; an already tinted surface turns towards it.
fn cast(c: Color32, hue: f32, chroma: f32) -> Color32 {
    if c.a() == 0 {
        return c;
    }
    let o = Oklch::from_srgb8([c.r(), c.g(), c.b()]);
    let [r, g, b] = Oklch::new(o.l, o.c.max(chroma), hue).to_srgb8();
    Color32::from_rgba_unmultiplied(r, g, b, c.a())
}

/// The theme's tokens shifted for `tint`.
pub fn tinted(base: Tokens, tint: &Tint) -> Tokens {
    let strength = 0.55 + 0.45 * tint.confidence.clamp(0.0, 1.0);
    // Surfaces: a whisper of the picture's hue, stronger when the picture is vivid; light
    // themes take a little less so text contrast is untouched.
    let vivid = (tint.chroma / 0.2).clamp(0.0, 1.0);
    let mut chroma = (0.010 + 0.022 * vivid) * strength;
    if !base.dark() {
        chroma *= 0.7;
    }
    let mut t = base;
    for c in [
        &mut t.chrome,
        &mut t.dock,
        &mut t.card,
        &mut t.card_border,
        &mut t.field,
        &mut t.field_border,
        &mut t.hover,
        &mut t.pressed,
        &mut t.separator,
        &mut t.tab_strip,
        &mut t.row_selected,
        &mut t.canvas,
        &mut t.canvas_dot,
        &mut t.accent_soft,
    ] {
        *c = cast(*c, tint.hue, chroma);
    }
    // The accent follows a clear hue; a grey or muddy picture keeps the theme's.
    if tint.chroma > 0.06 && tint.confidence > 0.25 {
        for c in [&mut t.accent, &mut t.accent_border] {
            let o = Oklch::from_srgb8([c.r(), c.g(), c.b()]);
            let [r, g, b] = Oklch::new(o.l, o.c, tint.hue).to_srgb8();
            *c = Color32::from_rgba_unmultiplied(r, g, b, c.a());
        }
    }
    t
}

/// Each frame: read the picture when it changed, keep the tokens and the UI font in step with
/// it, and return everything to the theme when there is nothing to follow.
pub fn sync(app: &mut BrahmaApp, ctx: &egui::Context) {
    let on = app.session.prefs().interface.adaptive_aesthetics;
    let now = ctx.input(|i| i.time);
    // 1. What to follow.
    let key = if on { app.session.active().map(|d| (d.doc.id, d.revision)) } else { None };
    if key != app.state_key() {
        app.adaptive.changed_at = Some(now);
        app.adaptive.key = key;
        if key.is_none() {
            app.adaptive.tint = None;
            app.adaptive.font = None;
        }
    }
    if let (Some(_), Some(at)) = (key, app.adaptive.changed_at)
        && now - at >= SETTLE_SECS
    {
        app.adaptive.changed_at = None;
        let tint = app.session.active().and_then(|d| tint_of(&d.doc));
        app.adaptive.tint = tint;
        app.adaptive.font = tint.and_then(|t| font_for(t.register)).map(str::to_string);
        if let Some(f) = &app.adaptive.font {
            brahma_text::served::request(f);
        }
    } else if app.adaptive.changed_at.is_some() {
        ctx.request_repaint_after(std::time::Duration::from_millis(120));
    }
    // 2. Tokens: re-apply whenever something else (a theme switch) put the plain ones back.
    let base = Tokens::for_kind(app.ui.theme);
    let want = match &app.adaptive.tint {
        Some(t) => tinted(base, t),
        None => base,
    };
    if Tokens::get(ctx) != want && (app.adaptive.tint.is_some() || Tokens::get(ctx).kind == want.kind) {
        theme::apply_tokens(ctx, want);
    }
    // 3. The UI font.
    for (family, bytes) in std::mem::take(&mut *ARRIVED.lock().unwrap_or_else(PoisonError::into_inner)) {
        app.adaptive.fonts.entry(family).or_insert(bytes);
    }
    let wanted = app.adaptive.font.clone().filter(|f| app.adaptive.fonts.contains_key(f)).map(|f| format!("{FONT_PREFIX}{f}"));
    if wanted != app.adaptive.installed {
        if app.adaptive.installed.take().is_some() {
            remove_ui_font(ctx);
        }
        if let Some(name) = wanted
            && let Some(bytes) = app.adaptive.font.as_ref().and_then(|f| app.adaptive.fonts.get(f)).cloned()
        {
            install_ui_font(ctx, &name, bytes);
            app.adaptive.installed = Some(name);
        }
        ctx.request_repaint();
    }
}

impl BrahmaApp {
    fn state_key(&self) -> Option<(DocId, u64)> {
        self.adaptive.key
    }
}

/// Register `bytes` as the first font of the proportional family and of the named weight
/// families (a variable font gets the weight through its `wght` axis).
fn install_ui_font(ctx: &egui::Context, name: &str, bytes: Arc<Vec<u8>>) {
    let mut data = FontData::from_owned((*bytes).clone());
    theme::size_ui_font(ctx, name, &mut data);
    let mut families = vec![InsertFontFamily { family: FontFamily::Proportional, priority: FontPriority::Highest }];
    for weight in ["medium", "semibold"] {
        families.push(InsertFontFamily { family: FontFamily::Name(weight.into()), priority: FontPriority::Highest });
    }
    ctx.add_font(FontInsert { name: name.to_string(), data, families });
}

/// Drop every adaptive font from the definitions (the theme's stack stays as installed).
fn remove_ui_font(ctx: &egui::Context) {
    let mut defs = ctx.fonts(|f| f.definitions().clone());
    let names: Vec<String> = defs.font_data.keys().filter(|k| k.starts_with(FONT_PREFIX)).cloned().collect();
    if names.is_empty() {
        return;
    }
    for n in &names {
        defs.font_data.remove(n);
    }
    for stack in defs.families.values_mut() {
        stack.retain(|n| !names.contains(n));
    }
    ctx.set_fonts(defs);
    // `set_fonts` drops the on-demand script fallbacks: let them load again.
    crate::cjk_fonts::install(ctx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;
    use serde_json::json;

    fn hue_of(c: Color32) -> Oklch {
        Oklch::from_srgb8([c.r(), c.g(), c.b()])
    }

    #[test]
    fn a_tint_casts_the_surfaces_and_turns_the_accent_only_for_a_clear_hue() {
        let base = Tokens::for_kind(theme::ThemeKind::Pro);
        let blue = Tint { hue: 250.0, chroma: 0.15, confidence: 0.8, register: Register::Trust };
        let t = tinted(base, &blue);
        for (name, c) in [("chrome", t.chrome), ("card", t.card), ("canvas", t.canvas), ("field", t.field)] {
            let o = hue_of(c);
            assert!((o.h - 250.0).abs() < 8.0 && o.c > 0.008, "{name} took the hue: {o:?}");
            let b = hue_of(match name {
                "chrome" => base.chrome,
                "card" => base.card,
                "canvas" => base.canvas,
                _ => base.field,
            });
            assert!((o.l - b.l).abs() < 0.02, "{name} keeps its lightness");
        }
        assert_eq!(t.text, base.text, "text never moves");
        assert!((hue_of(t.accent).h - 250.0).abs() < 8.0, "a vivid picture turns the accent");
        let grey = Tint { hue: 40.0, chroma: 0.01, confidence: 0.9, register: Register::Calm };
        let g = tinted(base, &grey);
        assert_eq!(g.accent, base.accent, "a grey picture keeps the theme's accent");
        assert_ne!(g.chrome, base.chrome, "but the surfaces still take a whisper of it");
    }

    #[test]
    fn registers_map_to_the_research_body_faces() {
        assert_eq!(font_for(Register::Neutral), None);
        assert_eq!(font_for(Register::Calm), None, "calm reads in the theme's own humanist face");
        assert_eq!(font_for(Register::Trust), Some("Source Sans 3"));
        assert_eq!(font_for(Register::Playful), Some("Nunito Sans"));
        for f in [font_for(Register::Trust), font_for(Register::Playful)].into_iter().flatten() {
            assert!(brahma_text::served::google_catalog().iter().any(|c| c.family == f), "{f} is in the catalogue");
        }
    }

    /// Opening a vivid document tints the interface after edits settle; closing it restores
    /// the theme; the preference turns it off.
    #[test]
    fn a_document_tints_the_interface_and_closing_it_restores_the_theme() {
        let fetched: Arc<Mutex<Vec<String>>> = Arc::default();
        let log = fetched.clone();
        let services = crate::Services {
            fetch_url: Some(Box::new(move |url, _integrity, reply| {
                log.lock().unwrap().push(url);
                // Any real font will do for the UI swap: the bundled Inter stands in.
                reply(Ok(brahma_text::fonts::INTER_REGULAR.to_vec()));
            })),
            ..Default::default()
        };
        let app = BrahmaApp::new(brahma_engine::Session::new(), services);
        let mut h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).with_step_dt(0.1).build_ui_state(
            |ui, app: &mut BrahmaApp| {
                sync(app, ui.ctx());
                egui::CentralPanel::default().show(ui, |_| {});
            },
            app,
        );
        BrahmaApp::setup_context(&h.ctx, Default::default());
        h.run_steps(2);
        let base = Tokens::for_kind(h.state().ui.theme);
        assert_eq!(Tokens::get(&h.ctx), base);
        // A saturated magenta document, playful by its chroma.
        h.state_mut().run("file.new", json!({"width": 64, "height": 64, "background": "#ff40c0"})).unwrap();
        h.run_steps(8);
        let tint = h.state().adaptive.tint.expect("read after settling");
        assert!(tint.chroma > 0.1, "{tint:?}");
        let t = Tokens::get(&h.ctx);
        assert_ne!(t, base);
        assert!((hue_of(t.chrome).h - tint.hue).abs() < 10.0, "chrome follows the picture: {:?} vs {}", hue_of(t.chrome), tint.hue);
        if let Some(font) = h.state().adaptive.font.clone() {
            assert!(fetched.lock().unwrap().iter().any(|u| u.contains("/ofl/")), "the face was fetched");
            h.run_steps(3);
            let first = h.ctx.fonts(|f| f.definitions().families[&FontFamily::Proportional].first().cloned());
            assert_eq!(first.as_deref(), Some(format!("{FONT_PREFIX}{font}").as_str()));
        }
        // Off: everything returns to the theme while the document stays open.
        h.state_mut().run("prefs.set", json!({"values": {"interface.adaptiveAesthetics": false}})).unwrap();
        h.run_steps(3);
        assert_eq!(Tokens::get(&h.ctx), base);
        assert!(h.state().adaptive.tint.is_none());
        let first = h.ctx.fonts(|f| f.definitions().families[&FontFamily::Proportional].first().cloned());
        assert!(!first.unwrap_or_default().starts_with(FONT_PREFIX), "the UI font is the theme's again");
        // On again, then close the document: back to the theme too.
        h.state_mut().run("prefs.set", json!({"values": {"interface.adaptiveAesthetics": true}})).unwrap();
        h.run_steps(8);
        assert_ne!(Tokens::get(&h.ctx), base);
        h.state_mut().run("file.close", json!({"force": true})).unwrap();
        h.run_steps(3);
        assert_eq!(Tokens::get(&h.ctx), base);
    }
}
