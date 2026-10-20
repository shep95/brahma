//! Window › Aesthetic Advisor: recommendations from the design-research corpus
//! (`brahma-aesthetic` through the `aesthetic.*` commands): a colour scheme in oklch for the
//! chosen emotional register and base colour, type pairing and scale, placement and
//! measurement, motion timing, and the research the advice rests on. "read the document"
//! analyses the open image's statistics and lets them pick the register and the base colour.
//! Swatches apply to the foreground colour with a click.

use egui::{Color32, RichText, vec2};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::theme::Tokens;
use crate::{BrahmaApp, widgets};

/// View state of the advisor window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AestheticUi {
    pub open: bool,
    /// `neutral`, `trust`, `calm`, `urgency`, `wonder`, `playful`.
    pub register: String,
    pub surface: String,
    pub density: String,
    pub touch: bool,
    /// Build the palette around the foreground colour (else the register's own hue).
    pub from_foreground: bool,
    /// The last recommendation (the `aesthetic.recommend` result), kept across frames.
    #[serde(skip)]
    pub result: Option<Value>,
    #[serde(skip)]
    pub analysis: Option<Value>,
    #[serde(skip)]
    pub error: Option<String>,
    /// The document and revision the analysis was read from: the window re-reads the picture
    /// when it opens on a different one.
    #[serde(skip)]
    pub read_key: Option<(brahma_doc::DocId, u64)>,
}

impl Default for AestheticUi {
    fn default() -> Self {
        Self {
            open: false,
            register: "neutral".into(),
            surface: "dark".into(),
            density: "comfortable".into(),
            touch: false,
            from_foreground: true,
            result: None,
            analysis: None,
            error: None,
            read_key: None,
        }
    }
}

const REGISTERS: [(&str, &str); 6] =
    [("neutral", "Neutral"), ("trust", "Trust"), ("calm", "Calm"), ("urgency", "Urgency"), ("wonder", "Wonder"), ("playful", "Playful")];
const SURFACES: [(&str, &str); 2] = [("dark", "Dark"), ("light", "Light")];
const DENSITIES: [(&str, &str); 3] = [("dense", "Dense"), ("comfortable", "Comfortable"), ("airy", "Airy")];

fn request(app: &BrahmaApp, from_document: bool) -> Value {
    let a = &app.ui.aesthetic;
    let mut p = json!({"register": a.register, "surface": a.surface, "density": a.density, "touch": a.touch});
    if a.from_foreground {
        p["base"] = json!("foreground");
    }
    if from_document {
        p["fromDocument"] = json!(true);
        // The document's own register wins over "neutral".
        if a.register == "neutral" {
            p.as_object_mut().map(|o| o.remove("register"));
        }
    }
    p
}

fn run(app: &mut BrahmaApp, from_document: bool) {
    if from_document {
        match app.run("aesthetic.analyze", json!({})) {
            Ok(v) => app.ui.aesthetic.analysis = Some(v),
            Err(e) => {
                app.ui.aesthetic.error = Some(e);
                return;
            }
        }
    }
    let p = request(app, from_document);
    match app.run("aesthetic.recommend", p) {
        Ok(v) => {
            app.ui.aesthetic.result = Some(v);
            app.ui.aesthetic.error = None;
        }
        Err(e) => app.ui.aesthetic.error = Some(e),
    }
}

pub fn window(app: &mut BrahmaApp, ctx: &egui::Context) {
    if !app.ui.aesthetic.open {
        return;
    }
    // The open picture is read on its own the first time the window sees it (and again
    // when a different document is in front), so the advice is always about this photo.
    let key = app.session.active().map(|d| (d.doc.id, d.revision));
    if let Some(k) = key
        && app.ui.aesthetic.read_key.is_none_or(|r| r.0 != k.0)
    {
        app.ui.aesthetic.read_key = Some(k);
        run(app, true);
    }
    let t = Tokens::get(ctx);
    let mut close = false;
    let mut action: Option<&'static str> = None;
    let mut apply: Option<usize> = None;
    let mut pick: Option<[f32; 3]> = None;
    let mut state = app.ui.aesthetic.clone();
    let width = 360.0;
    crate::analysis_ui::panel_window(app, ctx, "aesthetic-advisor", tl!("Aesthetic Advisor"), vec2(-20.0, 20.0), width, |ui| {
        close = crate::analysis_ui::title_row(ui, tl!("Aesthetic Advisor"));
        ui.horizontal_wrapped(|ui| {
            ui.label(tl!("Feeling"));
            widgets::dropdown(ui, "aesthetic-register", &mut state.register, &REGISTERS.map(|(k, l)| (k.to_string(), l)), 110.0);
            widgets::dropdown(ui, "aesthetic-surface", &mut state.surface, &SURFACES.map(|(k, l)| (k.to_string(), l)), 80.0);
            widgets::dropdown(ui, "aesthetic-density", &mut state.density, &DENSITIES.map(|(k, l)| (k.to_string(), l)), 110.0);
        });
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut state.from_foreground, tl!("Build around the foreground color"));
            ui.checkbox(&mut state.touch, tl!("Touch screen"));
        });
        ui.horizontal(|ui| {
            if widgets::primary_button(ui, tl!("Recommend"), 0.0).clicked() {
                action = Some("recommend");
            }
            let has_doc = ui.ctx().data(|d| d.get_temp::<bool>(egui::Id::new("aesthetic-has-doc"))).unwrap_or(false);
            if ui
                .add_enabled(has_doc, egui::Button::new(tl!("Read the document")))
                .on_hover_text(tl!("Measure the open image's lightness, contrast, edges and hues, and recommend from what it already says"))
                .clicked()
            {
                action = Some("document");
            }
        });
        if let Some(e) = &state.error {
            ui.colored_label(t.danger, e);
        }
        if let Some(a) = &state.analysis {
            widgets::hairline(ui);
            widgets::section_label(ui, "The document reads as");
            ui.label(
                RichText::new(format!("{}  ·  {:.0} %", a["register"].as_str().unwrap_or("?"), a["confidence"].as_f64().unwrap_or(0.0) * 100.0)).color(t.text),
            );
            for n in a["notes"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                ui.label(RichText::new(n).color(t.text_dim).size(11.5));
            }
            // Concrete edits for this picture, each one click away (an adjustment layer or a
            // filter), with the measurement and the principle behind it.
            let edits: Vec<&Value> = a["edits"].as_array().into_iter().flatten().collect();
            widgets::hairline(ui);
            widgets::section_label(ui, "Edits for this photo");
            if edits.is_empty() {
                ui.label(RichText::new(tl!("Balanced as it is: nothing the measurements would change.")).color(t.text_dim).size(11.5));
            }
            for (i, e) in edits.iter().enumerate() {
                ui.horizontal(|ui| {
                    if widgets::secondary_button(ui, tl!("Apply"), 0.0)
                        .on_hover_text(format!("{} {}", e["command"].as_str().unwrap_or(""), e["params"]))
                        .clicked()
                    {
                        apply = Some(i);
                    }
                    ui.label(RichText::new(e["title"].as_str().unwrap_or("")).color(t.text).font(crate::theme::medium(12.0)));
                });
                ui.label(RichText::new(e["why"].as_str().unwrap_or("")).color(t.text_dim).size(11.0));
                ui.add_space(2.0);
            }
        }
        let Some(r) = &state.result else { return };
        egui::ScrollArea::vertical().max_height(520.0).auto_shrink([false, true]).show(ui, |ui| {
            widgets::hairline(ui);
            widgets::section_label(ui, "Color scheme");
            ui.label(RichText::new(r["palette"]["harmony"].as_str().unwrap_or("")).color(t.text_dim).size(11.5));
            egui::Grid::new("aesthetic-palette").spacing(vec2(8.0, 4.0)).show(ui, |ui| {
                for s in r["palette"]["swatches"].as_array().into_iter().flatten() {
                    let hex = s["hex"].as_str().unwrap_or("#000000");
                    let rgb = brahma_aesthetic::parse_hex(hex).unwrap_or([0, 0, 0]);
                    let color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                    let role = s["role"].as_str().unwrap_or("");
                    if widgets::color_swatch_button(ui, color, &format!("{role}: {hex}")).clicked() {
                        pick = Some([f32::from(rgb[0]) / 255.0, f32::from(rgb[1]) / 255.0, f32::from(rgb[2]) / 255.0]);
                    }
                    ui.label(RichText::new(role).color(t.text).font(crate::theme::medium(12.0)));
                    ui.label(RichText::new(hex).color(t.text_dim).monospace().size(11.0));
                    ui.label(RichText::new(s["why"].as_str().unwrap_or("")).color(t.text_dim).size(11.0));
                    ui.end_row();
                }
            });
            ui.label(RichText::new(tl!("Click a swatch to make it the foreground color")).color(t.text_faint).size(10.5));

            widgets::hairline(ui);
            widgets::section_label(ui, "Type");
            let ty = &r["typography"];
            ui.label(
                RichText::new(format!("{}  +  {}", ty["heading"].as_str().unwrap_or(""), ty["body"].as_str().unwrap_or("")))
                    .color(t.text)
                    .font(crate::theme::medium(12.5)),
            );
            ui.label(RichText::new(ty["pairingWhy"].as_str().unwrap_or("")).color(t.text_dim).size(11.5));
            let sizes: Vec<String> = ty["sizesPx"].as_array().into_iter().flatten().filter_map(Value::as_f64).map(|v| format!("{v:.1}")).collect();
            ui.label(RichText::new(format!("{}  →  {} px", ty["scaleName"].as_str().unwrap_or(""), sizes.join(" / "))).color(t.text_dim).size(11.5));
            ui.label(
                RichText::new(format!(
                    "line height {:.2}  ·  {} to {} characters a line",
                    ty["lineHeight"].as_f64().unwrap_or(1.5),
                    ty["lineLengthChars"][0],
                    ty["lineLengthChars"][1]
                ))
                .color(t.text_dim)
                .size(11.5),
            );
            for n in ty["notes"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                ui.label(RichText::new(format!("· {n}")).color(t.text_dim).size(11.0));
            }

            widgets::hairline(ui);
            widgets::section_label(ui, "Placement and measurement");
            let l = &r["layout"];
            ui.label(
                RichText::new(format!(
                    "{} px grid  ·  targets ≥ {} px  ·  {} px inside groups, {} px between  ·  {} px margins",
                    l["gridPx"], l["minTargetPx"], l["gapRelatedPx"], l["gapGroupsPx"], l["marginPx"]
                ))
                .color(t.text)
                .size(11.5),
            );
            ui.label(RichText::new(format!("primary action: {}", l["primaryAction"].as_str().unwrap_or(""))).color(t.text_dim).size(11.5));
            ui.label(RichText::new(format!("shadows: {}", l["shadow"].as_str().unwrap_or(""))).color(t.text_dim).size(11.5));
            for n in l["notes"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                ui.label(RichText::new(format!("· {n}")).color(t.text_dim).size(11.0));
            }

            widgets::hairline(ui);
            widgets::section_label(ui, "Motion");
            let m = &r["motion"];
            ui.label(
                RichText::new(format!(
                    "{} / {} / {} ms  ·  enter {}  ·  exit {}",
                    m["microMs"],
                    m["standardMs"],
                    m["largeMs"],
                    m["enterEasing"].as_str().unwrap_or(""),
                    m["exitEasing"].as_str().unwrap_or("")
                ))
                .color(t.text)
                .size(11.5),
            );
            ui.label(RichText::new(format!("spring: {}", m["spring"].as_str().unwrap_or(""))).color(t.text_dim).size(11.5));
            for n in m["notes"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                ui.label(RichText::new(format!("· {n}")).color(t.text_dim).size(11.0));
            }

            widgets::hairline(ui);
            widgets::section_label(ui, "From the research");
            for e in r["principles"].as_array().into_iter().flatten() {
                ui.label(RichText::new(e["title"].as_str().unwrap_or("")).color(t.text).font(crate::theme::medium(11.5)));
                ui.label(RichText::new(e["summary"].as_str().unwrap_or("")).color(t.text_dim).size(11.0));
                ui.add_space(2.0);
            }
        });
    });
    // The window body can't borrow the app: apply what it decided afterwards.
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("aesthetic-has-doc"), app.session.active().is_some()));
    let changed = state != app.ui.aesthetic;
    app.ui.aesthetic = state;
    if changed {
        ctx.request_repaint();
    }
    if close {
        app.ui.aesthetic.open = false;
    }
    match action {
        Some("recommend") => run(app, false),
        Some("document") => run(app, true),
        _ => {}
    }
    if let Some(i) = apply {
        let edit = app.ui.aesthetic.analysis.as_ref().and_then(|a| a["edits"].get(i)).cloned();
        if let Some(e) = edit
            && let Some(id) = e["command"].as_str()
        {
            match app.run(id, e["params"].clone()) {
                Ok(_) => {
                    app.ui.status = format!("{}: {}", tl!("Applied"), e["title"].as_str().unwrap_or(id));
                    app.ui.status_error = false;
                }
                Err(err) => app.ui.status = err,
            }
            // The picture changed: read it again so the list reflects the new state.
            app.ui.aesthetic.read_key = None;
        }
    }
    if let Some(rgb) = pick
        && let Err(e) = app.run("tools.setColors", json!({"foreground": [rgb[0], rgb[1], rgb[2], 1.0]}))
    {
        app.ui.status = e;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    fn harness() -> Harness<'static, BrahmaApp> {
        let mut h = Harness::builder().with_size(vec2(1200.0, 900.0)).build_ui_state(
            |ui, app: &mut BrahmaApp| {
                egui::CentralPanel::default().show(ui, |_| {});
                window(app, ui.ctx());
            },
            BrahmaApp::new(brahma_engine::Session::new(), crate::Services::default()),
        );
        BrahmaApp::setup_context(&h.ctx, Default::default());
        h.run_steps(2);
        h
    }

    #[test]
    fn the_window_menu_opens_the_advisor_and_recommend_fills_it() {
        let mut h = harness();
        assert!(h.query_by_label("Recommend").is_none(), "closed by default");
        let ctx = h.ctx.clone();
        crate::menus::invoke(h.state_mut(), &ctx, "window.toggle.aestheticAdvisor", json!({})).unwrap();
        h.run_steps(2);
        assert!(h.state().ui.aesthetic.open);
        h.get_by_label("Recommend").click();
        h.run_steps(3);
        let r = h.state().ui.aesthetic.result.clone().expect("a recommendation");
        assert_eq!(r["palette"]["swatches"].as_array().map(Vec::len), Some(11));
        // "Read the document" is off without one; the swatches apply to the foreground.
        assert!(h.get_by_label("Read the document").accesskit_node().is_disabled());
        let hex = r["palette"]["swatches"][5]["hex"].as_str().unwrap().to_string();
        h.get_by_label(&format!("brand: {hex}")).click();
        h.run_steps(2);
        let fg = h.state().session.tools.foreground;
        let rgb = brahma_aesthetic::parse_hex(&hex).unwrap();
        assert!((fg[0] - f32::from(rgb[0]) / 255.0).abs() < 0.01, "{fg:?} vs {hex}");
        crate::menus::invoke(h.state_mut(), &ctx, "window.toggle.aestheticAdvisor", json!({})).unwrap();
        h.run_steps(2);
        assert!(!h.state().ui.aesthetic.open);
    }

    /// Opening the window on a document reads it by itself, lists the edits the picture would
    /// benefit from, and Apply runs one (an adjustment layer appears); the list then re-reads.
    #[test]
    fn the_window_reads_the_open_photo_itself_and_applies_an_edit() {
        let mut h = harness();
        h.state_mut().run("file.new", json!({"width": 48, "height": 48, "background": "#262628"})).unwrap();
        h.state_mut().ui.aesthetic.open = true;
        h.run_steps(3);
        let a = h.state().ui.aesthetic.analysis.clone().expect("read on open");
        let edits = a["edits"].as_array().cloned().unwrap_or_default();
        assert!(!edits.is_empty(), "a dark flat grey picture has edits: {a}");
        assert!(h.query_by_label("Edits for this photo").is_some());
        let layers_before = h.state().session.active().unwrap().doc.layers.len();
        h.get_all_by_label("Apply").next().expect("an apply button").click();
        h.run_steps(3);
        assert_eq!(h.state().session.active().unwrap().doc.layers.len(), layers_before + 1, "an adjustment layer was added");
        assert!(h.state().ui.status.starts_with("Applied"), "{}", h.state().ui.status);
        assert!(h.state().ui.aesthetic.read_key.is_some(), "read again after the edit");
    }

    #[test]
    fn reading_a_document_sets_the_register_from_its_statistics() {
        let mut h = harness();
        h.state_mut().ui.aesthetic.open = true;
        h.state_mut().run("file.new", json!({"width": 64, "height": 64, "background": "white"})).unwrap();
        h.run_steps(3);
        h.get_by_label("Read the document").click();
        h.run_steps(3);
        let a = h.state().ui.aesthetic.analysis.clone().expect("an analysis");
        assert_eq!(a["register"], "calm");
        let r = h.state().ui.aesthetic.result.clone().expect("a recommendation");
        assert_eq!(r["request"]["register"], "calm");
    }
}
