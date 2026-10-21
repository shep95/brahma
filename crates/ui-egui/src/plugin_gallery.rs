//! Filter › Plug-ins › Gallery…: sample WebAssembly plug-ins bundled with the app (built from
//! `examples/plugins/*-rs`, the template for writing your own), each one click from installed,
//! plus "install from a URL" for plug-ins published anywhere on the web. Installing goes through
//! `plugin.install` with the module's bytes, so it works the same on the desktop and in the
//! browser; the URL fetch goes through `Services::fetch_url`, the same path the fonts use.

use std::sync::mpsc::{Receiver, channel};

use egui::{RichText, vec2};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::theme::Tokens;
use crate::{BrahmaApp, widgets};

/// A plug-in shipped inside the app, as the gallery lists it.
pub struct Sample {
    pub id: &'static str,
    pub name: &'static str,
    pub what: &'static str,
    pub source: &'static str,
    pub bytes: &'static [u8],
}

/// The bundled samples, in gallery order.
pub const SAMPLES: &[Sample] = &[
    Sample {
        id: "org.brahma.example.invert",
        name: "Invert",
        what: "Inverts the colour channels. The smallest possible plug-in: no parameters, under a kilobyte.",
        source: "examples/plugins/invert-rs",
        bytes: include_bytes!("../../../assets/plugins/invert.wasm"),
    },
    Sample {
        id: "org.brahma.example.posterize",
        name: "Posterize",
        what: "Snaps each channel to a few levels. Shows a parameter (levels) read from the host's JSON.",
        source: "examples/plugins/posterize-rs",
        bytes: include_bytes!("../../../assets/plugins/posterize.wasm"),
    },
    Sample {
        id: "org.brahma.example.vignette",
        name: "Vignette",
        what: "Darkens towards the corners. Shows a position-dependent filter that stays seamless across the host's bands.",
        source: "examples/plugins/vignette-rs",
        bytes: include_bytes!("../../../assets/plugins/vignette.wasm"),
    },
];

/// Largest module fetched from a URL (the host's own module limit).
const MAX_FETCH_BYTES: usize = 32 * 1024 * 1024;

/// A module fetch in flight: the URL and where its bytes (or the failure) arrive.
type Fetching = (String, Receiver<Result<Vec<u8>, String>>);

/// View state of the gallery window.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginGalleryUi {
    pub open: bool,
    /// The URL typed into "install from a URL".
    pub url: String,
    #[serde(skip)]
    pub status: Option<(String, bool)>,
    /// A fetch in flight: (url, where the bytes arrive).
    #[serde(skip)]
    pub fetching: Option<Fetching>,
}

impl Clone for PluginGalleryUi {
    fn clone(&self) -> Self {
        Self { open: self.open, url: self.url.clone(), status: self.status.clone(), fetching: None }
    }
}

impl std::fmt::Debug for PluginGalleryUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginGalleryUi").field("open", &self.open).field("url", &self.url).finish()
    }
}

impl PartialEq for PluginGalleryUi {
    fn eq(&self, o: &Self) -> bool {
        self.open == o.open && self.url == o.url && self.status == o.status
    }
}

/// Install `bytes` as a plug-in; the status line says how it went.
fn install(app: &mut BrahmaApp, bytes: &[u8], from: &str) {
    use base64::Engine as _;
    let data = base64::engine::general_purpose::STANDARD.encode(bytes);
    match app.run("plugin.install", json!({"data": data, "replace": true})) {
        Ok(v) => {
            let name = v.get("name").and_then(serde_json::Value::as_str).unwrap_or(from);
            app.ui.plugin_gallery.status = Some((format!("{}: {name}", tl!("Installed")), false));
            app.ui.status = format!("{}: {name}", tl!("Installed"));
            app.ui.status_error = false;
        }
        Err(e) => {
            app.ui.plugin_gallery.status = Some((format!("{from}: {e}"), true));
        }
    }
}

/// Start fetching the module at `url` (https only).
fn fetch(app: &mut BrahmaApp, ctx: &egui::Context, url: &str) {
    let url = url.trim().to_string();
    if !url.starts_with("https://") {
        app.ui.plugin_gallery.status = Some((tl!("The address must start with https://").to_string(), true));
        return;
    }
    let Some(fetch) = app.services.fetch_url.as_ref() else {
        app.ui.plugin_gallery.status = Some((tl!("This build has no network access for plug-ins.").to_string(), true));
        return;
    };
    let (tx, rx) = channel();
    let repaint = ctx.clone();
    fetch(
        url.clone(),
        None,
        Box::new(move |r| {
            let _ = tx.send(r);
            repaint.request_repaint();
        }),
    );
    app.ui.plugin_gallery.fetching = Some((url, rx));
    app.ui.plugin_gallery.status = Some((tl!("Fetching…").to_string(), false));
}

/// Take a finished fetch and install it.
pub fn poll(app: &mut BrahmaApp, ctx: &egui::Context) {
    let Some((url, rx)) = app.ui.plugin_gallery.fetching.as_ref() else { return };
    let outcome = match rx.try_recv() {
        Ok(o) => o,
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
            return;
        }
        Err(std::sync::mpsc::TryRecvError::Disconnected) => Err("the request was dropped".into()),
    };
    let url = url.clone();
    app.ui.plugin_gallery.fetching = None;
    match outcome {
        Ok(bytes) if bytes.len() > MAX_FETCH_BYTES => app.ui.plugin_gallery.status = Some((format!("{url}: the module is too large"), true)),
        Ok(bytes) => install(app, &bytes, &url),
        Err(e) => app.ui.plugin_gallery.status = Some((format!("{url}: {e}"), true)),
    }
}

pub fn window(app: &mut BrahmaApp, ctx: &egui::Context) {
    if !app.ui.plugin_gallery.open {
        return;
    }
    poll(app, ctx);
    let t = Tokens::get(ctx);
    let installed: Vec<String> = brahma_plugins::registry::list().iter().map(|p| p.manifest().id.clone()).collect();
    let mut close = false;
    let mut install_sample: Option<usize> = None;
    let mut install_url = false;
    let mut url = app.ui.plugin_gallery.url.clone();
    let status = app.ui.plugin_gallery.status.clone();
    let busy = app.ui.plugin_gallery.fetching.is_some();
    crate::analysis_ui::panel_window(app, ctx, "plugin-gallery", tl!("Plug-in Gallery"), vec2(-20.0, 60.0), 400.0, |ui| {
        close = crate::analysis_ui::title_row(ui, tl!("Plug-in Gallery"));
        ui.label(RichText::new(tl!("Sandboxed WebAssembly filters: one file runs on every platform and in the browser.")).color(t.text_dim).size(11.5));
        widgets::hairline(ui);
        widgets::section_label(ui, "Samples");
        for (i, s) in SAMPLES.iter().enumerate() {
            ui.horizontal(|ui| {
                let is_in = installed.iter().any(|id| id == s.id);
                let label = if is_in { tl!("Installed") } else { tl!("Install") };
                if ui.add_enabled(!is_in, egui::Button::new(label)).clicked() {
                    install_sample = Some(i);
                }
                ui.label(RichText::new(s.name).color(t.text).font(crate::theme::medium(12.0)));
                ui.label(RichText::new(format!("{} B", s.bytes.len())).color(t.text_faint).size(10.5));
            });
            ui.label(RichText::new(s.what).color(t.text_dim).size(11.0));
            ui.label(RichText::new(format!("{}: {}", tl!("source"), s.source)).color(t.text_faint).monospace().size(10.5));
            ui.add_space(3.0);
        }
        widgets::hairline(ui);
        widgets::section_label(ui, "From the web");
        ui.horizontal(|ui| {
            widgets::text_field(ui, egui::Id::new("plugin-gallery-url"), &mut url, "https://…/plugin.wasm", 270.0, true);
            if ui.add_enabled(!busy && !url.trim().is_empty(), egui::Button::new(tl!("Install"))).clicked() {
                install_url = true;
            }
        });
        ui.label(RichText::new(tl!("Any .wasm that follows the plug-in ABI; it is checked before it runs and can touch nothing but its pixels.")).color(t.text_dim).size(11.0));
        if let Some((text, error)) = &status {
            ui.label(RichText::new(text).color(if *error { t.danger } else { t.accent }).size(11.5));
        }
        widgets::hairline(ui);
        widgets::section_label(ui, "Write your own");
        ui.label(RichText::new(tl!("Copy a sample folder, edit the manifest and pc_filter, run examples/plugins/build-all.sh, then install the .wasm here or from Filter › Plug-ins › Install Plug-in…. The ABI is in docs/plugins.md.")).color(t.text_dim).size(11.0));
    });
    if url != app.ui.plugin_gallery.url {
        app.ui.plugin_gallery.url = url.clone();
    }
    if close {
        app.ui.plugin_gallery.open = false;
    }
    if let Some(i) = install_sample
        && let Some(s) = SAMPLES.get(i)
    {
        install(app, s.bytes, s.name);
    }
    if install_url {
        fetch(app, ctx, &url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    use std::sync::{Arc, Mutex};

    fn harness(fetch: Option<crate::FetchFn>) -> Harness<'static, BrahmaApp> {
        let services = crate::Services { fetch_url: fetch, ..Default::default() };
        let mut app = BrahmaApp::new(brahma_engine::Session::new(), services);
        app.session.execute("file.new", json!({"width": 24, "height": 16, "background": "#80c0ff"})).unwrap();
        app.sync_views();
        let mut h = Harness::builder().with_size(vec2(1100.0, 800.0)).build_ui_state(
            |ui, app: &mut BrahmaApp| {
                egui::CentralPanel::default().show(ui, |_| {});
                window(app, ui.ctx());
            },
            app,
        );
        BrahmaApp::setup_context(&h.ctx, Default::default());
        h.run_steps(1);
        h.state_mut().ui.plugin_gallery.open = true;
        h.run_steps(2);
        h
    }

    /// Every bundled sample installs with one click and runs on the document (with its
    /// parameters validated by the host); the installed ones read "Installed".
    #[test]
    fn samples_install_with_one_click_and_run() {
        for s in SAMPLES {
            let _ = brahma_plugins::registry::remove(s.id);
        }
        let mut h = harness(None);
        let installs = h.get_all_by_label("Install").count();
        assert!(installs >= SAMPLES.len(), "{installs}");
        for _ in 0..SAMPLES.len() {
            h.get_all_by_label("Install").next().expect("an install button").click();
            h.run_steps(2);
        }
        let ids: Vec<String> = brahma_plugins::registry::list().iter().map(|p| p.manifest().id.clone()).collect();
        for s in SAMPLES {
            assert!(ids.contains(&s.id.to_string()), "{} installed: {ids:?}", s.id);
        }
        assert!(h.state().ui.plugin_gallery.status.as_ref().is_some_and(|(m, e)| m.starts_with("Installed") && !e), "{:?}", h.state().ui.plugin_gallery.status);
        let px = |app: &BrahmaApp| app.session.active().unwrap().doc.layers[0].surface().unwrap().pixel(1, 1);
        let before = px(h.state());
        h.state_mut().run("plugin.run", json!({"id": "org.brahma.example.posterize", "levels": 2})).unwrap();
        let after = px(h.state());
        assert!(after.iter().take(3).all(|v| *v == 0.0 || *v == 1.0), "two levels: {after:?} (was {before:?})");
        h.state_mut().run("plugin.run", json!({"id": "org.brahma.example.vignette", "amount": 100})).unwrap();
        let corner = px(h.state());
        let centre = h.state().session.active().unwrap().doc.layers[0].surface().unwrap().pixel(12, 8);
        assert!(corner[2] < centre[2], "the corner is darker than the centre: {corner:?} vs {centre:?}");
        assert!(h.state_mut().run("plugin.run", json!({"id": "org.brahma.example.posterize", "levels": "many"})).is_err(), "a wrong type is refused by the host");
        for s in SAMPLES {
            let _ = brahma_plugins::registry::remove(s.id);
        }
    }

    /// A URL goes through the fetch service and installs what comes back; non-https and a
    /// failed fetch show in the status line without a crash.
    #[test]
    fn a_url_installs_through_the_fetch_service() {
        let _ = brahma_plugins::registry::remove("org.brahma.example.invert");
        let asked: Arc<Mutex<Vec<String>>> = Arc::default();
        let seen = asked.clone();
        let fetch: crate::FetchFn = Box::new(move |url, _integrity, reply| {
            seen.lock().unwrap().push(url.clone());
            if url.ends_with("invert.wasm") { reply(Ok(SAMPLES[0].bytes.to_vec())) } else { reply(Err("404".into())) }
        });
        let mut h = harness(Some(fetch));
        h.state_mut().ui.plugin_gallery.url = "http://insecure/x.wasm".into();
        h.run_steps(1);
        h.get_all_by_label("Install").last().unwrap().click();
        h.run_steps(2);
        assert!(asked.lock().unwrap().is_empty(), "http is refused before any request");
        assert!(h.state().ui.plugin_gallery.status.as_ref().is_some_and(|(_, e)| *e));
        h.state_mut().ui.plugin_gallery.url = "https://example.test/missing.wasm".into();
        h.run_steps(1);
        h.get_all_by_label("Install").last().unwrap().click();
        h.run_steps(3);
        assert!(h.state().ui.plugin_gallery.status.as_ref().is_some_and(|(m, e)| *e && m.contains("404")), "{:?}", h.state().ui.plugin_gallery.status);
        h.state_mut().ui.plugin_gallery.url = "https://example.test/invert.wasm".into();
        h.run_steps(1);
        h.get_all_by_label("Install").last().unwrap().click();
        h.run_steps(3);
        assert!(brahma_plugins::registry::get("org.brahma.example.invert").is_some(), "{:?}", h.state().ui.plugin_gallery.status);
        assert_eq!(asked.lock().unwrap().len(), 2);
        let _ = brahma_plugins::registry::remove("org.brahma.example.invert");
    }
}
