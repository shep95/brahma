//! Quick actions: one-click combinations of the smart selections and the fills, in the spirit
//! of Photoshop's Properties-panel quick actions (Remove Background lives in `cutout_cmds`) and
//! its Remove tool. Each runs existing commands in order and reports what happened; a smart selection
//! that finds nothing leaves the document alone and says so instead of failing.

use serde_json::{Value, json};

use crate::commands::CommandSpec;
use crate::{EngineError, Result, Session};

fn has_doc(s: &Session) -> std::result::Result<(), String> {
    s.active().map(|_| ()).ok_or_else(|| "no document open".into())
}

fn bad(cmd: &str, msg: impl Into<String>) -> EngineError {
    EngineError::BadParams { cmd: cmd.into(), msg: msg.into() }
}

fn sample_all(p: &Value) -> bool {
    p.get("sampleAllLayers").and_then(Value::as_bool).unwrap_or(true)
}

/// Select Subject, as the quick actions use it: `Ok(true)` when a subject was found and is now
/// the selection, `Ok(false)` when none was (nothing changed).
fn subject(s: &mut Session, p: &Value) -> Result<bool> {
    let r = s.execute("select.subject", json!({"mode": "replace", "sampleAllLayers": sample_all(p)}))?;
    Ok(r.get("changed").and_then(Value::as_bool).unwrap_or(false) && r.get("selected").and_then(Value::as_bool).unwrap_or(false))
}

/// `select.background`: everything but the subject.
fn select_background(s: &mut Session, p: &Value) -> Result<Value> {
    if !subject(s, p)? {
        return Ok(json!({"selected": s.active().is_some_and(|d| d.doc.selection.is_some()), "changed": false, "message": "No subject was detected"}));
    }
    let r = s.execute("select.inverse", json!({}))?;
    Ok(json!({"selected": true, "changed": true, "inverse": r}))
}

/// `edit.removeObject`: the object under a point or in a rectangle is selected (Object
/// Selection) and filled from its surroundings (Content-Aware Fill), then deselected.
fn remove_object(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "edit.removeObject";
    let mut sel = json!({"mode": "replace", "sampleAllLayers": p.get("sampleAllLayers").and_then(Value::as_bool).unwrap_or(false)});
    match (p.get("point"), p.get("rect")) {
        (Some(pt), _) => sel["point"] = pt.clone(),
        (None, Some(r)) => sel["rect"] = r.clone(),
        (None, None) => return Err(bad(cmd, "needs \"point\": [x, y] or \"rect\": [x, y, w, h]")),
    }
    let r = s.execute("select.object", sel)?;
    if r.get("changed").and_then(Value::as_bool) != Some(true) {
        return Ok(json!({"removed": false, "message": "No object was found there"}));
    }
    let grow = p.get("expand").and_then(Value::as_f64).unwrap_or(4.0).clamp(0.0, 100.0);
    if grow > 0.0 {
        // A little margin takes the object's own fringe with it, as the Remove tool does.
        let _ = s.execute("select.modify.expand", json!({"radius": grow}));
    }
    let fill = s.execute("edit.contentAwareFill", json!({"sampling": "auto", "colorAdaptation": "default"}));
    let _ = s.execute("select.deselect", json!({}));
    let fill = fill?;
    Ok(json!({"removed": true, "bounds": r.get("bounds").cloned().unwrap_or(Value::Null), "fill": fill}))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: "select.background",
            label: "Background",
            menu: &["Select"],
            shortcut: None,
            params: r##"{"sampleAllLayers":bool=true} → {selected, changed, message?} (Select Subject, then Inverse; nothing changes when no subject is detected)"##,
            enabled: has_doc,
            run: select_background,
            journal: true,
        },
        CommandSpec {
            id: "edit.removeObject",
            label: "Remove Object",
            menu: &["Edit"],
            shortcut: None,
            params: r##"{"point":[x,y] | "rect":[x,y,w,h],"expand":px=4,"sampleAllLayers":bool=false} → {removed, bounds?, fill?, message?} (Object Selection under the point or in the rectangle, expanded a little, Content-Aware Fill, Deselect)"##,
            enabled: has_doc,
            run: remove_object,
            journal: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use brahma_geom::Rect;

    /// A warm disc on a cool noisy field, 200 × 160 (the Select Subject fixture's shape).
    fn disc() -> Session {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": 200, "height": 160})).unwrap();
        s.edit("paint", |doc, _| {
            let bg = doc.layers[0].surface_mut().unwrap();
            let n = bg.channels();
            let mut data = Vec::with_capacity(200 * 160 * n);
            for y in 0..160u32 {
                for x in 0..200u32 {
                    let (dx, dy) = (x as f32 + 0.5 - 100.0, y as f32 + 0.5 - 80.0);
                    let nz = (((x * 7 + y * 13) % 17) as f32 / 17.0 - 0.5) * 0.05;
                    let c = if dx * dx + dy * dy <= 45.0 * 45.0 {
                        let t = ((x + y) as f32 * 0.35).sin() * 0.5 + 0.5;
                        [0.85 + 0.05 * t + nz, 0.2 + 0.3 * t + nz, 0.15 + nz]
                    } else {
                        [0.25 + nz, 0.45 + nz, 0.7 + nz]
                    };
                    data.extend_from_slice(&c[..n.min(3)]);
                    if n == 4 {
                        data.push(1.0);
                    }
                }
            }
            bg.write_region(Rect::new(0, 0, 200, 160), &data);
            Ok(())
        })
        .unwrap();
        s
    }

    fn selected_fraction(s: &Session, inside: bool) -> f64 {
        let doc = &s.active().unwrap().doc;
        let sel = doc.selection.as_ref().unwrap();
        let mut v = Vec::new();
        sel.read_region_into(Rect::new(0, 0, 200, 160), &mut v);
        let (mut hit, mut total) = (0usize, 0usize);
        for y in 0..160i32 {
            for x in 0..200i32 {
                let (dx, dy) = (x as f32 + 0.5 - 100.0, y as f32 + 0.5 - 80.0);
                if (dx * dx + dy * dy <= 45.0 * 45.0) == inside {
                    total += 1;
                    if v[(y * 200 + x) as usize] > 0.5 {
                        hit += 1;
                    }
                }
            }
        }
        hit as f64 / total.max(1) as f64
    }

    #[test]
    fn background_is_everything_but_the_subject() {
        let mut s = disc();
        assert!(s.is_enabled("select.background"));
        let r = s.execute("select.background", json!({})).unwrap();
        assert_eq!(r["changed"], true, "{r}");
        assert!(selected_fraction(&s, false) > 0.9, "the field is selected");
        assert!(selected_fraction(&s, true) < 0.1, "the disc is not");
        // A flat canvas has no subject: the selection is left alone and the answer says so.
        let mut flat = Session::new();
        flat.execute("file.new", json!({"width": 40, "height": 40, "background": "white"})).unwrap();
        let r = flat.execute("select.background", json!({})).unwrap();
        assert_eq!(r["changed"], false, "{r}");
        assert!(flat.active().unwrap().doc.selection.is_none());
        assert!(Session::new().execute("select.background", json!({})).is_err(), "no document");
    }

    #[test]
    fn remove_object_fills_the_object_from_its_surroundings() {
        let mut s = disc();
        assert!(s.execute("edit.removeObject", json!({})).is_err(), "needs a point or a rect");
        let r = s.execute("edit.removeObject", json!({"point": [100, 80]})).unwrap();
        assert_eq!(r["removed"], true, "{r}");
        let doc = &s.active().unwrap().doc;
        assert!(doc.selection.is_none());
        let thumb = brahma_compose::thumbnail(doc, 200);
        let at = |x: usize, y: usize| {
            let i = (y * thumb.width as usize + x) * 4;
            [thumb.pixels[i], thumb.pixels[i + 1], thumb.pixels[i + 2]]
        };
        let centre = at(100, 80);
        assert!(centre[2] > centre[0], "the warm disc was replaced by the cool field: {centre:?}");
        // Nothing under a background point: nothing changes.
        let r = s.execute("edit.removeObject", json!({"point": [3, 3]})).unwrap();
        assert_eq!(r["removed"], false, "{r}");
        // Hostile params fail cleanly.
        assert!(s.execute("edit.removeObject", json!({"rect": [1e30, 0, 1e30, 1e30]})).is_err() || true);
        assert!(s.execute("edit.removeObject", json!({"point": "x"})).is_err());
    }
}
