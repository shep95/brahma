//! Aesthetic recommendations (`brahma-aesthetic`): colour schemes, type, placement and
//! measurement, motion timing, from the design-research corpus; and the analysis of the active
//! document's statistics (lightness, contrast, edge density, chroma, hue spread) that predicts
//! the emotional register a viewer feels before reading it. Queries only: nothing here edits
//! a document, so nothing is journaled.

use brahma_aesthetic::{Analysis, Density, Register, Request, Surface, parse_hex};
use serde_json::{Value, json};

use crate::commands::{CommandSpec, always};
use crate::{EngineError, Result, Session};

fn bad(what: &str, msg: impl Into<String>) -> EngineError {
    EngineError::Other(format!("{what}: {}", msg.into()))
}

/// Build a request from params: `register`, `base` (`#rrggbb`, or `"foreground"` for the tool
/// colour), `surface` (`dark` | `light`), `density` (`dense` | `comfortable` | `airy`), `touch`.
fn request(s: &Session, p: &Value) -> Result<Request> {
    let mut req = Request::default();
    if let Some(r) = p.get("register") {
        let name = r.as_str().ok_or_else(|| bad("aesthetic", "`register` must be a string"))?;
        req.register =
            Register::parse(name).ok_or_else(|| bad("aesthetic", format!("unknown register `{name}`: neutral, trust, calm, urgency, wonder or playful")))?;
    }
    if let Some(b) = p.get("base") {
        req.base = match b {
            Value::Null => None,
            Value::String(hex) if hex == "foreground" => {
                let fg = s.tools.foreground;
                Some([
                    (fg[0].clamp(0.0, 1.0) * 255.0).round() as u8,
                    (fg[1].clamp(0.0, 1.0) * 255.0).round() as u8,
                    (fg[2].clamp(0.0, 1.0) * 255.0).round() as u8,
                ])
            }
            Value::String(hex) => Some(parse_hex(hex).ok_or_else(|| bad("aesthetic", format!("`base` is not a colour: {hex}")))?),
            Value::Array(a) if a.len() >= 3 => {
                let ch = |i: usize| a.get(i).and_then(Value::as_f64).map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
                Some([
                    ch(0).ok_or_else(|| bad("aesthetic", "`base` channels must be numbers 0..1"))?,
                    ch(1).ok_or_else(|| bad("aesthetic", "`base` channels must be numbers 0..1"))?,
                    ch(2).ok_or_else(|| bad("aesthetic", "`base` channels must be numbers 0..1"))?,
                ])
            }
            _ => return Err(bad("aesthetic", "`base` must be `#rrggbb`, [r, g, b] in 0..1, or \"foreground\"")),
        };
    }
    if let Some(v) = p.get("surface") {
        req.surface = match v.as_str() {
            Some("dark") => Surface::Dark,
            Some("light") => Surface::Light,
            _ => return Err(bad("aesthetic", "`surface` must be dark or light")),
        };
    }
    if let Some(v) = p.get("density") {
        req.density = match v.as_str() {
            Some("dense") => Density::Dense,
            Some("comfortable") => Density::Comfortable,
            Some("airy") => Density::Airy,
            _ => return Err(bad("aesthetic", "`density` must be dense, comfortable or airy")),
        };
    }
    if let Some(v) = p.get("touch") {
        req.touch = v.as_bool().ok_or_else(|| bad("aesthetic", "`touch` must be true or false"))?;
    }
    Ok(req)
}

fn recommend(s: &mut Session, p: &Value) -> Result<Value> {
    let req = request(s, p)?;
    let from_document = p.get("fromDocument").and_then(Value::as_bool).unwrap_or(false);
    let rec = if from_document {
        let a = analysis_of_active(s)?;
        brahma_aesthetic::recommend_for_image(req, &a)
    } else {
        brahma_aesthetic::recommend(&req)
    };
    serde_json::to_value(rec).map_err(|e| bad("aesthetic.recommend", e.to_string()))
}

/// The active document's statistics on a 256 px thumbnail.
fn analysis_of_active(s: &Session) -> Result<Analysis> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let thumb = brahma_compose::thumbnail(&d.doc, 256);
    brahma_aesthetic::analyze(thumb.width, thumb.height, &thumb.pixels).ok_or_else(|| bad("aesthetic.analyze", "the document has no opaque pixels to analyse"))
}

fn analyze(s: &mut Session, _p: &Value) -> Result<Value> {
    let a = analysis_of_active(s)?;
    let mut v = serde_json::to_value(&a).map_err(|e| bad("aesthetic.analyze", e.to_string()))?;
    if let Some(o) = v.as_object_mut() {
        o.insert("dominantHex".into(), json!(a.dominant.hex()));
        o.insert("register".into(), json!(a.register.name()));
        // Edits the picture itself would benefit from, each a runnable command.
        o.insert("edits".into(), serde_json::to_value(brahma_aesthetic::edit_suggestions(&a)).unwrap_or(Value::Null));
    }
    Ok(v)
}

fn search(_s: &mut Session, p: &Value) -> Result<Value> {
    let query = p.get("query").and_then(Value::as_str).ok_or_else(|| bad("aesthetic.search", "`query` (a string) is required"))?;
    let limit = p.get("limit").and_then(Value::as_u64).unwrap_or(8).clamp(1, 50) as usize;
    let hits: Vec<Value> = brahma_aesthetic::search(query, limit)
        .into_iter()
        .map(|(score, e)| json!({"score": score, "title": e.title, "domain": e.domain, "summary": e.summary, "tags": e.tags}))
        .collect();
    Ok(json!({"query": query, "entries": hits}))
}

/// The name of the layer group a look lives in.
pub fn look_group_name(id: &str) -> String {
    format!("look: {id}")
}

/// Every look group in the document (top level), as `(group id, look id, visible)`.
pub fn applied_looks(doc: &brahma_doc::Document) -> Vec<(brahma_doc::LayerId, String, bool)> {
    doc.layers
        .iter()
        .filter(|l| l.is_group())
        .filter_map(|l| l.name.strip_prefix("look: ").map(|id| (l.id, id.to_string(), l.visible)))
        .collect()
}

/// `aesthetic.look`: apply a look from the catalogue as one group of adjustment layers at the
/// top of the stack, one history step. A look already applied is replaced (its group is
/// rebuilt), so clicking twice does not stack it. Returns the group.
fn apply_look(s: &mut Session, p: &Value) -> Result<Value> {
    let id = p.get("look").and_then(Value::as_str).ok_or_else(|| bad("aesthetic.look", "`look` (a look id, see aesthetic.looks) is required"))?;
    let look = brahma_aesthetic::look(id).ok_or_else(|| {
        bad("aesthetic.look", format!("unknown look `{id}`: {}", brahma_aesthetic::looks().iter().map(|l| l.id).collect::<Vec<_>>().join(", ")))
    })?;
    let mode = s.active().ok_or(EngineError::NoDocument)?.doc.mode;
    let mut layers = Vec::with_capacity(look.steps.len());
    for step in &look.steps {
        let adj = crate::adjust_params::from_params(step.kind, &step.params, None, mode)?;
        let name = adj.label().to_string();
        layers.push(brahma_doc::Layer::new(name, brahma_doc::LayerContent::Adjustment(adj)));
    }
    let group_name = look_group_name(look.id);
    let label = format!("Look: {}", look.title);
    let gid = s.edit(&label, |doc, active| {
        let old: Vec<brahma_doc::LayerId> = doc.layers.iter().filter(|l| l.is_group() && l.name == group_name).map(|l| l.id).collect();
        for id in old {
            doc.remove(id);
        }
        let top = doc.layers.last().map(|l| l.id);
        let gid = doc.insert_above(top, brahma_doc::Layer::group(group_name.clone(), layers));
        *active = Some(gid);
        Ok(gid)
    })?;
    crate::layer_multi_cmds::reselect(s, vec![gid], Some(gid));
    Ok(json!({"layer": gid.0, "look": look.id, "title": look.title, "steps": look.steps.len()}))
}

/// `aesthetic.looks`: the catalogue, each with its fit for the active document (when one is
/// open), and the looks already applied to it.
fn list_looks(s: &mut Session, _p: &Value) -> Result<Value> {
    let catalogue = serde_json::to_value(brahma_aesthetic::looks()).map_err(|e| bad("aesthetic.looks", e.to_string()))?;
    let (suggested, applied) = match s.active() {
        Some(d) => {
            let applied: Vec<Value> = applied_looks(&d.doc).into_iter().map(|(gid, id, visible)| json!({"layer": gid.0, "look": id, "visible": visible})).collect();
            let suggested = analysis_of_active(s).ok().map(|a| brahma_aesthetic::look_suggestions(&a)).unwrap_or_default();
            (serde_json::to_value(suggested).unwrap_or(Value::Null), Value::Array(applied))
        }
        None => (Value::Null, json!([])),
    };
    Ok(json!({"looks": catalogue, "suggested": suggested, "applied": applied}))
}

/// The group of the look `p["look"]` (or the newest applied look without one).
fn look_group(s: &Session, p: &Value, cmd: &str) -> Result<brahma_doc::LayerId> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let applied = applied_looks(&d.doc);
    match p.get("look").and_then(Value::as_str) {
        Some(id) => applied.iter().find(|(_, l, _)| l == id.trim()).map(|(g, _, _)| *g).ok_or_else(|| bad(cmd, format!("the look `{id}` is not applied"))),
        None => applied.last().map(|(g, _, _)| *g).ok_or_else(|| bad(cmd, "no look is applied")),
    }
}

/// `aesthetic.toggleLook`: hide or show a look's group (the before/after switch).
fn toggle_look(s: &mut Session, p: &Value) -> Result<Value> {
    let gid = look_group(s, p, "aesthetic.toggleLook")?;
    let want = p.get("visible").and_then(Value::as_bool);
    let visible = s.edit("Toggle Look", |doc, _| {
        let l = doc.layer_mut(gid).ok_or(EngineError::NoLayer(gid))?;
        l.visible = want.unwrap_or(!l.visible);
        Ok(l.visible)
    })?;
    Ok(json!({"layer": gid.0, "visible": visible}))
}

/// `aesthetic.removeLook`: delete a look's group.
fn remove_look(s: &mut Session, p: &Value) -> Result<Value> {
    let gid = look_group(s, p, "aesthetic.removeLook")?;
    s.edit("Remove Look", |doc, active| {
        doc.remove(gid).ok_or(EngineError::NoLayer(gid))?;
        if *active == Some(gid) {
            *active = doc.layers.last().map(|l| l.id);
        }
        Ok(())
    })?;
    let active = s.active().and_then(|d| d.doc.layers.last().map(|l| l.id));
    crate::layer_multi_cmds::reselect(s, active.into_iter().collect(), active);
    Ok(json!({"layer": gid.0}))
}

fn has_doc(s: &Session) -> std::result::Result<(), String> {
    s.active().map(|_| ()).ok_or_else(|| "no document open".into())
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: "aesthetic.looks",
            label: "Looks",
            menu: &[],
            shortcut: None,
            params: r##"{} → {looks:[{id, title, register, why, steps}], suggested:[{id, title, fit, why}] (for the active document), applied:[{layer, look, visible}]}"##,
            enabled: always,
            run: list_looks,
            journal: false,
        },
        CommandSpec {
            id: "aesthetic.look",
            label: "Apply Look",
            menu: &[],
            shortcut: None,
            params: r##"{"look": id (see aesthetic.looks)} → {layer (the group), look, title, steps}; one group of adjustment layers named "look: <id>" at the top of the stack, replacing the same look if already applied"##,
            enabled: has_doc,
            run: apply_look,
            journal: true,
        },
        CommandSpec {
            id: "aesthetic.toggleLook",
            label: "Toggle Look (Before/After)",
            menu: &[],
            shortcut: None,
            params: r##"{"look?": id (default: the newest applied look), "visible?": bool (default: flip)} → {layer, visible}"##,
            enabled: has_doc,
            run: toggle_look,
            journal: true,
        },
        CommandSpec {
            id: "aesthetic.removeLook",
            label: "Remove Look",
            menu: &[],
            shortcut: None,
            params: r##"{"look?": id (default: the newest applied look)} → {layer}"##,
            enabled: has_doc,
            run: remove_look,
            journal: true,
        },
        CommandSpec {
            id: "aesthetic.recommend",
            label: "Aesthetic Recommendations",
            menu: &[],
            shortcut: None,
            params: r##"{"register?": "neutral|trust|calm|urgency|wonder|playful", "base?": "#rrggbb | [r,g,b] | \"foreground\"", "surface?": "dark|light", "density?": "dense|comfortable|airy", "touch?": bool, "fromDocument?": bool}"##,
            enabled: always,
            run: recommend,
            journal: false,
        },
        CommandSpec {
            id: "aesthetic.analyze",
            label: "Analyze Aesthetic Register",
            menu: &[],
            shortcut: None,
            params: r##"{}"##,
            enabled: has_doc,
            run: analyze,
            journal: false,
        },
        CommandSpec {
            id: "aesthetic.search",
            label: "Search Design Research",
            menu: &[],
            shortcut: None,
            params: r##"{"query": string, "limit?": 1..50}"##,
            enabled: always,
            run: search,
            journal: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommend_answers_with_every_family_and_rejects_bad_params() {
        let mut s = Session::new();
        let v = s.execute("aesthetic.recommend", json!({"register": "calm", "surface": "dark", "density": "dense", "touch": true})).unwrap();
        assert_eq!(v["palette"]["swatches"].as_array().map(Vec::len), Some(11));
        assert_eq!(v["typography"]["basePx"], json!(16.0));
        assert!(v["layout"]["minTargetPx"].as_u64().unwrap() >= 44);
        assert!(v["principles"].as_array().is_some_and(|p| !p.is_empty()));
        for bad in [
            json!({"register": "spicy"}),
            json!({"base": "#zz"}),
            json!({"surface": "grey"}),
            json!({"density": 3}),
            json!({"touch": "yes"}),
            json!({"base": [1]}),
        ] {
            assert!(s.execute("aesthetic.recommend", bad.clone()).is_err(), "{bad}");
        }
        // The foreground colour as the base.
        s.execute("tools.setColors", json!({"foreground": [0.79, 0.66, 0.35, 1.0]})).unwrap();
        let v = s.execute("aesthetic.recommend", json!({"base": "foreground"})).unwrap();
        assert_eq!(v["request"]["base"], json!([201, 168, 89]));
    }

    #[test]
    fn analyze_needs_a_document_and_reads_its_register() {
        let mut s = Session::new();
        assert!(s.execute("aesthetic.analyze", json!({})).is_err());
        s.execute("file.new", json!({"width": 64, "height": 64, "background": "white"})).unwrap();
        let v = s.execute("aesthetic.analyze", json!({})).unwrap();
        assert!(v["meanLightness"].as_f64().unwrap() > 0.95, "{v}");
        assert_eq!(v["register"], "calm");
        assert!(v["dominantHex"].as_str().unwrap().starts_with('#'));
        // Every suggested edit is a real command that runs on the document.
        s.execute("file.new", json!({"width": 48, "height": 48, "background": "#262628"})).unwrap();
        let v = s.execute("aesthetic.analyze", json!({})).unwrap();
        let edits = v["edits"].as_array().unwrap();
        assert!(!edits.is_empty(), "{v}");
        for e in edits {
            let id = e["command"].as_str().unwrap();
            assert!(crate::commands::find(id).is_some(), "{id}");
            s.execute(id, e["params"].clone()).unwrap_or_else(|err| panic!("{id} {}: {err}", e["params"]));
        }
        s.execute("file.close", json!({"force": true})).unwrap();
        let r = s.execute("aesthetic.recommend", json!({"fromDocument": true})).unwrap();
        assert_eq!(r["request"]["register"], "calm");
    }

    #[test]
    fn looks_apply_as_one_group_toggle_and_remove() {
        let mut s = Session::new();
        assert!(s.execute("aesthetic.look", json!({"look": "warm"})).is_err(), "no document");
        let v = s.execute("aesthetic.looks", json!({})).unwrap();
        assert!(v["looks"].as_array().unwrap().len() >= 6);
        assert!(v["suggested"].is_null() && v["applied"].as_array().unwrap().is_empty());
        s.execute("file.new", json!({"width": 32, "height": 32, "background": "#8a6a3a"})).unwrap();
        assert!(s.execute("aesthetic.look", json!({})).is_err());
        assert!(s.execute("aesthetic.look", json!({"look": "nope"})).is_err());
        // Every look applies on an RGB document, as one history step.
        for l in brahma_aesthetic::looks() {
            let before = s.active().unwrap().history.past_len();
            let v = s.execute("aesthetic.look", json!({"look": l.id})).unwrap();
            assert_eq!(v["steps"].as_u64(), Some(l.steps.len() as u64), "{}", l.id);
            let after = s.active().unwrap().history.past_len();
            assert_eq!(after, before + 1, "{}: one history step", l.id);
            assert_eq!(s.active().unwrap().history.undo_label(), Some(format!("Look: {}", l.title).as_str()));
        }
        let v = s.execute("aesthetic.looks", json!({})).unwrap();
        let applied = v["applied"].as_array().unwrap();
        assert_eq!(applied.len(), brahma_aesthetic::looks().len(), "{v}");
        assert!(!v["suggested"].as_array().unwrap().is_empty());
        // Applying a look again replaces it rather than stacking.
        let n = s.active().unwrap().doc.layers.len();
        let g = s.execute("aesthetic.look", json!({"look": "warm"})).unwrap();
        assert_eq!(s.active().unwrap().doc.layers.len(), n);
        let group = s.active().unwrap().doc.layer(brahma_doc::LayerId(g["layer"].as_u64().unwrap())).unwrap().clone();
        assert_eq!(group.name, "look: warm");
        assert_eq!(group.children().map(<[brahma_doc::Layer]>::len), Some(3));
        // Before/after: the newest look (warm, just re-applied at the top) hides and shows.
        let t = s.execute("aesthetic.toggleLook", json!({})).unwrap();
        assert_eq!((t["layer"].as_u64(), t["visible"].as_bool()), (g["layer"].as_u64(), Some(false)));
        let t = s.execute("aesthetic.toggleLook", json!({"look": "warm", "visible": true})).unwrap();
        assert_eq!(t["visible"], true);
        assert!(s.execute("aesthetic.toggleLook", json!({"look": "nope"})).is_err());
        // Remove by name, then everything else by default order; a bad id is an error.
        s.execute("aesthetic.removeLook", json!({"look": "mono"})).unwrap();
        assert!(s.execute("aesthetic.removeLook", json!({"look": "mono"})).is_err());
        while !s.execute("aesthetic.looks", json!({})).unwrap()["applied"].as_array().unwrap().is_empty() {
            s.execute("aesthetic.removeLook", json!({})).unwrap();
        }
        assert!(s.execute("aesthetic.removeLook", json!({})).is_err());
        assert_eq!(s.active().unwrap().doc.layers.len(), 1);
        // Undo brings a look back whole.
        s.execute("aesthetic.look", json!({"look": "calm"})).unwrap();
        s.execute("edit.undo", json!({})).unwrap();
        assert_eq!(s.active().unwrap().doc.layers.len(), 1);
    }

    #[test]
    fn search_returns_ranked_entries_and_validates_its_query() {
        let mut s = Session::new();
        assert!(s.execute("aesthetic.search", json!({})).is_err());
        let v = s.execute("aesthetic.search", json!({"query": "dark mode halation", "limit": 3})).unwrap();
        let entries = v["entries"].as_array().unwrap();
        assert!(!entries.is_empty() && entries.len() <= 3);
        assert!(entries[0]["score"].as_f64().unwrap() >= entries.last().unwrap()["score"].as_f64().unwrap());
    }
}
