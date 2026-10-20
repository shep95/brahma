//! Commands on the document as a whole that have no menu of their own: renaming it (the
//! tab's name, which becomes the file name suggested on save).

use serde_json::{Value, json};

use crate::commands::CommandSpec;
use crate::{EngineError, Result, Session};

/// The longest document name accepted (file systems stop at 255 bytes).
const MAX_NAME: usize = 200;

fn has_doc(s: &Session) -> std::result::Result<(), String> {
    s.active().map(|_| ()).ok_or_else(|| "no document open".into())
}

/// A name a file system and a tab can take: trimmed, no path separators or control
/// characters, not empty, not longer than [`MAX_NAME`] characters.
pub fn clean_name(raw: &str) -> std::result::Result<String, String> {
    let name: String = raw.trim().chars().filter(|c| !c.is_control()).collect();
    if name.is_empty() {
        return Err("the name is empty".into());
    }
    if name.contains(['/', '\\', ':']) {
        return Err("the name can't contain / \\ or :".into());
    }
    if name.chars().count() > MAX_NAME {
        return Err(format!("the name is longer than {MAX_NAME} characters"));
    }
    Ok(name)
}

fn rename(s: &mut Session, p: &Value) -> Result<Value> {
    let bad = |msg: String| EngineError::BadParams { cmd: "document.rename".into(), msg };
    let raw = p.get("name").and_then(Value::as_str).ok_or_else(|| bad("`name` (a string) is required".into()))?;
    let name = clean_name(raw).map_err(bad)?;
    let index = match p.get("document") {
        None | Some(Value::Null) => s.active_index().ok_or(EngineError::NoDocument)?,
        Some(v) => {
            v.as_u64().and_then(|i| usize::try_from(i).ok()).filter(|i| *i < s.documents().len()).ok_or_else(|| bad("`document` out of range".into()))?
        }
    };
    let st = s.document_mut(index).ok_or(EngineError::NoDocument)?;
    let before = std::mem::replace(&mut std::sync::Arc::make_mut(&mut st.doc).name, name.clone());
    if before != name {
        // A new name is unsaved work: the file on disk still carries the old one.
        st.revision = st.revision.wrapping_add(1);
    }
    Ok(json!({"name": name, "was": before, "document": index}))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![CommandSpec {
        id: "document.rename",
        label: "Rename Document",
        menu: &[],
        shortcut: None,
        params: r##"{"name": string, "document?": index (the active one)}"##,
        enabled: has_doc,
        run: rename,
        journal: true,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renaming_changes_the_tab_name_marks_unsaved_and_rejects_bad_names() {
        let mut s = Session::new();
        assert!(s.execute("document.rename", json!({"name": "x"})).is_err(), "no document");
        s.execute("file.new", json!({"width": 8, "height": 8})).unwrap();
        let st = s.active_mut().unwrap();
        st.saved_revision = st.revision;
        assert!(!s.active().unwrap().is_dirty());
        let v = s.execute("document.rename", json!({"name": "  sunset over the field  "})).unwrap();
        assert_eq!(v["name"], "sunset over the field");
        assert_eq!(s.active().unwrap().doc.name, "sunset over the field");
        assert!(s.active().unwrap().is_dirty(), "a new name is unsaved work");
        for bad in [
            json!({}),
            json!({"name": ""}),
            json!({"name": "   "}),
            json!({"name": "a/b"}),
            json!({"name": "x".repeat(300)}),
            json!({"name": 5}),
            json!({"name": "ok", "document": 9}),
        ] {
            assert!(s.execute("document.rename", bad.clone()).is_err(), "{bad}");
        }
        assert_eq!(clean_name("tab\tname\n").unwrap(), "tabname");
        s.execute("file.new", json!({"width": 8, "height": 8})).unwrap();
        s.execute("document.rename", json!({"name": "first", "document": 0})).unwrap();
        assert_eq!(s.documents()[0].doc.name, "first");
    }
}
