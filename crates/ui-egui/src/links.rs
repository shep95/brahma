//! Community and project links: the Help menu, the About dialog, the start screen and the title
//! bar all open these. URLs follow the #houseofasher pattern (`github.com/shep95/brahma/apps/<app>`,
//! `github.com/storytold/<app>`).

use serde_json::{Value, json};

use crate::BrahmaApp;

pub const DISCORD: &str = "https://github.com/shep95/brahma/discussions";
pub const ARTCRAFT_WEBSITE: &str = "https://github.com/shep95/brahma";
pub const APP_PAGE: &str = "https://github.com/shep95/brahma";
pub const ISSUES: &str = "https://github.com/shep95/brahma/issues";

/// Help-menu link commands: (id, url). Labels live in `menus::UI_COMMANDS`.
pub const COMMANDS: &[(&str, &str)] =
    &[("help.discord", DISCORD), ("help.website", APP_PAGE), ("help.artcraftWebsite", ARTCRAFT_WEBSITE), ("help.reportIssue", ISSUES)];

pub fn url_for(id: &str) -> Option<&'static str> {
    COMMANDS.iter().find(|c| c.0 == id).map(|c| c.1)
}

/// Open `url` in the system browser (a new tab on the web) and note it in the status bar. Prefers
/// the platform `open_url` service (reliable on Windows/macOS/Linux); falls back to `ctx.open_url`.
pub fn open(app: &mut BrahmaApp, ctx: &egui::Context, url: &str) -> Value {
    // Only web links reach the system's URL handler: a `file:`, `javascript:` or custom-scheme
    // URL from any source would otherwise run whatever the OS registered for it.
    if !is_web_url(url) {
        app.ui.status = crate::i18n::fmt(tl!("Not a web link: {url}"), &[("url", url)]);
        return json!({"url": url, "opened": false});
    }
    let opened = app.services.open_url.as_ref().map(|f| f(url).is_ok()).unwrap_or(false);
    if !opened {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    let template = "Opened {url}";
    app.ui.status = crate::i18n::fmt(tl!(template), &[("url", url)]);
    json!({"url": url})
}

/// `http://` or `https://`, with a host: the only URLs [`open`] passes on.
pub fn is_web_url(url: &str) -> bool {
    let rest = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"));
    rest.is_some_and(|r| r.chars().next().is_some_and(|c| c.is_ascii_alphanumeric()))
}

/// The prominent "Join us in the #houseofasher community" button.
pub fn discord_button(app: &mut BrahmaApp, ui: &mut egui::Ui, min_width: f32) -> egui::Response {
    let r = crate::widgets::primary_button(ui, tl!("Join us in the #houseofasher community"), min_width).on_hover_text(DISCORD);
    if r.clicked() {
        open(app, ui.ctx(), DISCORD);
    }
    r
}

/// "brahma website · #houseofasher" as links, centred. Clicks route through [`open`] (the
/// platform browser service) rather than `ui.hyperlink_to`, which uses the unreliable `ctx.open_url`.
pub fn link_row(app: &mut BrahmaApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let links = [(tl!("brahma website"), APP_PAGE), (tl!("#houseofasher"), ARTCRAFT_WEBSITE)];
    let font = egui::FontId::proportional(12.5);
    let sep = "  ·  ";
    let width: f32 = links.iter().map(|(l, _)| ui.painter().layout_no_wrap((*l).into(), font.clone(), t.text).size().x).sum::<f32>()
        + 2.0 * ui.painter().layout_no_wrap(sep.into(), font.clone(), t.text).size().x;
    let mut clicked: Option<&str> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        for (i, (label, url)) in links.iter().enumerate() {
            if i > 0 {
                ui.label(egui::RichText::new(sep).font(font.clone()).color(t.text_faint));
            }
            if ui.link(egui::RichText::new(*label).font(font.clone()).color(t.accent)).on_hover_text(*url).clicked() {
                clicked = Some(url);
            }
        }
    });
    if let Some(url) = clicked {
        open(app, ui.ctx(), url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_web_links_are_opened() {
        assert!(is_web_url("https://github.com/shep95/brahma"));
        assert!(is_web_url("http://localhost:8765/"));
        for bad in ["file:///etc/passwd", "javascript:alert(1)", "ms-msdt:/id", "https://", "", "github.com/x"] {
            assert!(!is_web_url(bad), "{bad}");
        }
    }

    #[test]
    fn links_open_through_the_platform_service() {
        // Issue #14: links (Help menu, #houseofasher community button, start-page links) must open in the browser.
        // They route through the `open_url` service rather than the unreliable `ctx.open_url`.
        use std::sync::{Arc, Mutex};
        let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rec = opened.clone();
        let services = crate::Services {
            open_url: Some(Box::new(move |u: &str| {
                rec.lock().unwrap().push(u.to_string());
                Ok(())
            })),
            ..Default::default()
        };
        let mut app = BrahmaApp::new(brahma_engine::Session::new(), services);
        let ctx = egui::Context::default();
        // Every Help-menu link id reaches the service with its URL.
        for (id, url) in COMMANDS {
            crate::menus::invoke(&mut app, &ctx, id, json!({})).unwrap();
            assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(*url), "{id}");
        }
        // The direct open() helper (#houseofasher community button / start-page links) also uses it.
        open(&mut app, &ctx, DISCORD);
        assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(DISCORD));
    }

    #[test]
    fn help_menu_lists_links_then_separator_then_system_info_and_about() {
        let app = BrahmaApp::new(brahma_engine::Session::new(), Default::default());
        let help: Vec<String> = crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        assert_eq!(help, ["help.discord", "help.website", "help.artcraftWebsite", "help.reportIssue", "---", "help.systemInfo", "help.about"]);
        for (id, _) in COMMANDS {
            assert!(crate::menus::is_live(id) && crate::menus::is_enabled(&app, id), "{id}");
        }
    }

    #[test]
    fn link_commands_open_their_urls() {
        let mut app = BrahmaApp::new(brahma_engine::Session::new(), Default::default());
        let ctx = egui::Context::default();
        for (id, url) in [("help.discord", "https://github.com/shep95/brahma/discussions"), ("help.website", "https://github.com/shep95/brahma")] {
            let r = crate::menus::invoke(&mut app, &ctx, id, serde_json::json!({})).unwrap();
            assert_eq!(r["url"], url);
            assert_eq!(app.ui.status, format!("Opened {url}"));
        }
    }
}
