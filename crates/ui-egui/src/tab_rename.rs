//! Renaming a document from its tab: a double-click on the tab turns its name into a field,
//! Enter commits (`document.rename`), Escape cancels, and clicking elsewhere commits too. The
//! field owns the keyboard while it is up, so shortcuts don't fire on the letters typed.

use brahma_doc::DocId;
use egui::{FontId, Rect};
use serde_json::{Value, json};

#[derive(Clone, Debug)]
struct Rename {
    doc: DocId,
    text: String,
    focused: bool,
    /// The pass the field was last drawn in: a tab that scrolled out of view ends the rename.
    pass: u64,
}

fn id() -> egui::Id {
    egui::Id::new("brahma-tab-rename")
}

fn current(ctx: &egui::Context) -> Option<Rename> {
    ctx.data(|d| d.get_temp::<Rename>(id()))
}

fn store(ctx: &egui::Context, r: Option<Rename>) {
    ctx.data_mut(|d| match r {
        Some(r) => {
            d.insert_temp(id(), r);
        }
        None => {
            d.remove::<Rename>(id());
        }
    });
}

/// The document whose tab is being renamed.
pub fn renaming(ctx: &egui::Context) -> Option<DocId> {
    current(ctx).map(|r| r.doc)
}

/// Is a rename field on screen (this pass or the last)? It owns the keyboard meanwhile.
pub fn active(ctx: &egui::Context) -> bool {
    current(ctx).is_some_and(|r| r.pass + 1 >= ctx.cumulative_pass_nr())
}

/// Start renaming `doc` (now called `name`).
pub fn start(ctx: &egui::Context, doc: DocId, name: &str) {
    store(ctx, Some(Rename { doc, text: name.to_string(), focused: false, pass: ctx.cumulative_pass_nr() }));
}

/// Draw the field over `rect` when `doc` is the tab being renamed. Returns the command to run
/// when the rename was committed this frame: `(id, params)` with the document's index.
pub fn field(ui: &mut egui::Ui, doc: DocId, index: usize, rect: Rect, font: FontId) -> Option<(&'static str, Value)> {
    let ctx = ui.ctx().clone();
    let mut r = current(&ctx).filter(|r| r.doc == doc)?;
    let t = crate::theme::Tokens::get(&ctx);
    let field_id = egui::Id::new(("tab-rename-field", doc));
    ui.painter().rect_filled(rect, 0.0, t.field);
    let inner = rect.shrink2(egui::vec2(6.0, 3.0));
    let te = ui.put(inner, egui::TextEdit::singleline(&mut r.text).id(field_id).font(font).frame(egui::Frame::NONE).desired_width(inner.width()));
    r.pass = ctx.cumulative_pass_nr();
    if !r.focused {
        te.request_focus();
        if let Some(mut state) = egui::TextEdit::load_state(&ctx, field_id) {
            let all = egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(r.text.chars().count()));
            state.cursor.set_char_range(Some(all));
            state.store(&ctx, field_id);
        }
        r.focused = true;
        store(&ctx, Some(r));
        return None;
    }
    let (enter, esc) = ctx.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape)));
    if esc {
        store(&ctx, None);
        None
    } else if enter || te.lost_focus() || !te.has_focus() {
        store(&ctx, None);
        let name = r.text.trim().to_string();
        (!name.is_empty()).then(|| ("document.rename", json!({"name": name, "document": index})))
    } else {
        store(&ctx, Some(r));
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BrahmaApp;
    use egui_kittest::{Harness, kittest::Queryable};
    use serde_json::json;

    fn harness() -> Harness<'static, BrahmaApp> {
        let mut app = BrahmaApp::new(brahma_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"width": 32, "height": 32})).unwrap();
        app.run("file.new", json!({"width": 32, "height": 32})).unwrap();
        let mut h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_eframe(move |_| app);
        h.run_steps(3);
        h
    }

    /// Double-click a tab, type a name, Enter: the document is renamed and marked unsaved;
    /// Escape leaves the name alone. Both strips (pro and studio) share the field.
    #[test]
    fn double_clicking_a_tab_renames_the_document() {
        let mut h = harness();
        let doc0 = h.state().session.documents()[0].doc.id;
        let before = h.state().session.documents()[0].doc.name.clone();
        let tab = h.state().tab_strip.as_ref().and_then(|s| s.tabs.first().map(|(_, r)| *r)).expect("a tab");
        let at = egui::pos2(tab.left() + 40.0, tab.center().y);
        h.input_mut().events.push(egui::Event::PointerMoved(at));
        for _ in 0..2 {
            h.input_mut().events.push(egui::Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            });
            h.input_mut().events.push(egui::Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            });
            h.run_steps(1);
        }
        h.run_steps(2);
        assert_eq!(renaming(&h.ctx), Some(doc0), "double-click opens the field");
        assert!(active(&h.ctx));
        h.input_mut().events.push(egui::Event::Text("golden hour".into()));
        h.run_steps(1);
        h.input_mut().events.push(egui::Event::Key { key: egui::Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() });
        h.run_steps(2);
        assert_eq!(renaming(&h.ctx), None);
        let st = &h.state().session.documents()[0];
        assert_eq!(st.doc.name, "golden hour", "was {before}");
        assert!(st.is_dirty());
        assert!(h.query_by_label("golden hour @ 100% (RGB/8)*").is_some() || h.query_by_label("golden hour").is_some(), "the tab shows the new name");
        // Escape cancels.
        start(&h.ctx, doc0, "golden hour");
        h.run_steps(2);
        h.input_mut().events.push(egui::Event::Text("zzz".into()));
        h.run_steps(1);
        h.input_mut().events.push(egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() });
        h.run_steps(2);
        assert_eq!(h.state().session.documents()[0].doc.name, "golden hour");
        assert_eq!(renaming(&h.ctx), None);
    }
}
