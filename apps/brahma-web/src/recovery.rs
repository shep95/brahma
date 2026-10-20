//! Crash recovery in the browser: unsaved documents are autosaved as `.pcraft` bytes into
//! IndexedDB (database `brahma.recovery`, one record per document), and whatever is there when
//! the page next loads is offered back through the shell's recovery path (Preferences › File
//! Handling, `file.recover`), as the desktop app does with its recovery directory.
//!
//! The shell drives the timing (every N minutes, plus [`IdleFlush`]'s pause-after-edits on the
//! web, where a tab can be closed at any moment) and the bookkeeping of what is saved; this
//! module owns the records. Saving or closing a document drops its record.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Which record each open document writes to. Fresh documents get a key unique to this page
/// load (document ids restart at 1 every launch, so a plain id would overwrite another launch's
/// record before the user recovered it); a recovered document adopts the record it came from.
pub struct Keys {
    session: String,
    keys: HashMap<u64, String>,
}

impl Keys {
    pub fn new(session: impl Into<String>) -> Self {
        Self { session: session.into(), keys: HashMap::new() }
    }

    /// The record document `id` autosaves to.
    pub fn key_for(&mut self, id: u64) -> String {
        self.keys.entry(id).or_insert_with(|| format!("{}-{id}", self.session)).clone()
    }

    /// Recovered document `id` takes over record `key`.
    pub fn adopt(&mut self, id: u64, key: &str) {
        self.keys.insert(id, key.to_string());
    }

    /// Document `id` is saved or closed: the record to delete (none when it never autosaved).
    pub fn discard(&mut self, id: u64) -> Option<String> {
        self.keys.remove(&id)
    }
}

/// Autosave a pause after editing, not only on the interval timer: a browser tab is closed
/// far more casually than a desktop window, and a hidden tab gets no frames at all, so the
/// minutes-long interval alone would lose most of a session. The fingerprint is whatever the
/// caller derives from the dirty documents' revisions; a change starts the idle clock.
pub struct IdleFlush {
    fingerprint: u64,
    changed_at: f64,
    flushed: u64,
}

/// What [`IdleFlush::tick`] asks of the caller.
#[derive(Debug, PartialEq, Eq)]
pub enum Flush {
    /// Nothing changed since the last flush.
    Idle,
    /// Edits are settling: come back in this many milliseconds.
    After(u32),
    /// The pause is long enough: autosave now.
    Now,
}

impl IdleFlush {
    /// How long edits must have paused before they are flushed.
    pub const IDLE_MS: f64 = 5_000.0;

    pub fn new() -> Self {
        Self { fingerprint: 0, changed_at: 0.0, flushed: 0 }
    }

    pub fn tick(&mut self, fingerprint: u64, now: f64) -> Flush {
        if fingerprint != self.fingerprint {
            self.fingerprint = fingerprint;
            self.changed_at = now;
        }
        if fingerprint == self.flushed || fingerprint == 0 {
            return Flush::Idle;
        }
        let waited = now - self.changed_at;
        if waited >= Self::IDLE_MS {
            self.flushed = fingerprint;
            Flush::Now
        } else {
            Flush::After((Self::IDLE_MS - waited).clamp(1.0, Self::IDLE_MS) as u32)
        }
    }
}

impl Default for IdleFlush {
    fn default() -> Self {
        Self::new()
    }
}

/// One number for the set of dirty documents and their revisions (0 when nothing is dirty).
pub fn fingerprint(dirty: impl Iterator<Item = (u64, u64)>) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    let mut any = false;
    for (id, revision) in dirty {
        (id, revision).hash(&mut h);
        any = true;
    }
    if any { h.finish().max(1) } else { 0 }
}

#[cfg(target_arch = "wasm32")]
pub use web::{install, load};

#[cfg(target_arch = "wasm32")]
mod web {
    use super::Keys;
    use crate::indexed_presets::{TIMEOUT_MS, before, complete, js, open_store, request};
    use brahma_doc::Document;
    use brahma_ui_egui::{Recoverable, Services};
    use js_sys::{Array, Object, Reflect, Uint8Array};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Blob, IdbDatabase, IdbTransactionMode};

    const DATABASE: &str = "brahma.recovery";
    const STORE: &str = "documents";
    /// Largest record read back (a 1 GiB bundle is beyond what the page can hold anyway).
    const MAX_ENTRY_BYTES: f64 = 1024.0 * 1024.0 * 1024.0;
    /// Most bytes read back at one launch.
    const MAX_TOTAL_BYTES: u64 = 3 * 1024 * 1024 * 1024;
    const MAX_ENTRIES: usize = 64;

    /// A record found at launch: the `.pcraft` bytes of one unsaved document.
    pub struct Entry {
        pub key: String,
        pub name: String,
        pub path: Option<String>,
        pub bytes: Vec<u8>,
    }

    /// Open the store and read every record (bounded), before the app starts so recovery can
    /// be listed synchronously. `Err` when IndexedDB is unavailable or times out: the session
    /// still opens, without autosave.
    pub async fn load() -> Result<(IdbDatabase, Vec<Entry>), String> {
        let deadline = js_sys::Date::now() + TIMEOUT_MS;
        let db = open_store(DATABASE, STORE, deadline).await?;
        match read(&db, deadline).await {
            Ok(entries) => Ok((db, entries)),
            Err(e) => {
                db.close();
                Err(e)
            }
        }
    }

    fn field(obj: &JsValue, name: &str) -> Option<String> {
        Reflect::get(obj, &name.into()).ok().and_then(|v| v.as_string())
    }

    async fn read(db: &IdbDatabase, deadline: f64) -> Result<Vec<Entry>, String> {
        let tx = db.transaction_with_str_and_mode(STORE, IdbTransactionMode::Readonly).map_err(js)?;
        let req = tx.object_store(STORE).map_err(js)?.open_cursor().map_err(js)?;
        let mut records = Vec::new();
        let mut total = 0u64;
        let mut next = request(&req);
        loop {
            let value = before(&next.promise, deadline).await?;
            if value.is_null() {
                break;
            }
            let cursor = value.dyn_into::<web_sys::IdbCursorWithValue>().map_err(|_| "invalid recovery cursor")?;
            let key = cursor.key().map_err(js)?.as_string().unwrap_or_default();
            let record = cursor.value().map_err(js)?;
            let blob = Reflect::get(&record, &"data".into()).ok().and_then(|v| v.dyn_into::<Blob>().ok());
            match blob {
                Some(blob) if !key.is_empty() && blob.size() <= MAX_ENTRY_BYTES && records.len() < MAX_ENTRIES => {
                    total = total.saturating_add(blob.size() as u64);
                    if total > MAX_TOTAL_BYTES {
                        log::warn!("recovery: skipping {key}: too much recovery data to read");
                    } else {
                        let name = field(&record, "name").unwrap_or_else(|| "Recovered".into());
                        let path = field(&record, "path");
                        records.push((key, name, path, blob));
                    }
                }
                _ => log::warn!("recovery: skipping unreadable record {key}"),
            }
            // Register the next callback before continuing; blobs are read after enumeration.
            next = request(&req);
            cursor.continue_().map_err(js)?;
        }
        let mut entries = Vec::with_capacity(records.len());
        for (key, name, path, blob) in records {
            let buffer = before(&blob.array_buffer(), deadline).await?;
            entries.push(Entry { key, name, path, bytes: Uint8Array::new(&buffer).to_vec() });
        }
        Ok(entries)
    }

    async fn put(db: &IdbDatabase, key: &str, name: &str, path: Option<&str>, bytes: &[u8]) -> Result<(), String> {
        let tx = db.transaction_with_str_and_mode(STORE, IdbTransactionMode::Readwrite).map_err(js)?;
        let queued = (|| -> Result<(), String> {
            let store = tx.object_store(STORE).map_err(js)?;
            let record = Object::new();
            Reflect::set(&record, &"name".into(), &name.into()).map_err(js)?;
            Reflect::set(&record, &"path".into(), &path.map_or(JsValue::NULL, JsValue::from_str)).map_err(js)?;
            Reflect::set(&record, &"saved".into(), &js_sys::Date::now().into()).map_err(js)?;
            let blob = Blob::new_with_u8_array_sequence(&Array::of1(&Uint8Array::from(bytes))).map_err(js)?;
            Reflect::set(&record, &"data".into(), &blob).map_err(js)?;
            store.put_with_key(&record, &JsValue::from_str(key)).map_err(js)?;
            Ok(())
        })();
        if let Err(e) = queued {
            let _ = tx.abort();
            return Err(e);
        }
        finish(tx).await
    }

    async fn delete(db: &IdbDatabase, key: &str) -> Result<(), String> {
        let tx = db.transaction_with_str_and_mode(STORE, IdbTransactionMode::Readwrite).map_err(js)?;
        if let Err(e) = tx.object_store(STORE).map_err(js).and_then(|s| s.delete(&JsValue::from_str(key)).map_err(js)) {
            let _ = tx.abort();
            return Err(e);
        }
        finish(tx).await
    }

    async fn finish(tx: web_sys::IdbTransaction) -> Result<(), String> {
        let done = complete(&tx);
        let result = before(&done.promise, js_sys::Date::now() + TIMEOUT_MS).await.map(|_| ());
        if result.is_err() {
            let _ = tx.abort();
        }
        result
    }

    struct Store {
        db: Option<IdbDatabase>,
        keys: Keys,
        /// Finished writes for `autosave_results`: (document id, revision, outcome).
        results: Vec<(u64, u64, Result<(), String>)>,
        /// Records found at launch, handed to the shell once.
        entries: Vec<Entry>,
    }

    type Shared = Rc<RefCell<Store>>;

    fn with<R>(store: &Shared, f: impl FnOnce(&mut Store) -> R) -> Result<R, String> {
        let mut slot = store.try_borrow_mut().map_err(|_| "crash recovery is busy".to_string())?;
        Ok(f(&mut slot))
    }

    /// Wire the five recovery services into `services`. Without a database every autosave
    /// fails with a message the shell shows once per interval, and nothing is recovered.
    pub fn install(services: &mut Services, db: Option<IdbDatabase>, entries: Vec<Entry>, ctx: egui::Context) {
        let session = format!("{:x}{:x}", js_sys::Date::now() as u64, (js_sys::Math::random() * 1e9) as u64);
        let store: Shared = Rc::new(RefCell::new(Store { db, keys: Keys::new(session), results: Vec::new(), entries }));
        let (s1, s2, s3, s4) = (store.clone(), store.clone(), store.clone(), store.clone());
        services.autosave = Some(Box::new(move |doc: &Arc<Document>, revision: u64, path: Option<&str>| {
            let (db, key) = with(&s1, |s| s.db.clone().map(|db| (db, s.keys.key_for(doc.id.0))))?.ok_or("browser storage is unavailable")?;
            let bytes = brahma_format::save_to_bytes(doc, &brahma_format::SaveOptions::default()).map_err(|e| e.to_string())?;
            let (id, name, path) = (doc.id.0, doc.name.clone(), path.map(str::to_string));
            let (store, ctx) = (s1.clone(), ctx.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let result = put(&db, &key, &name, path.as_deref(), &bytes).await;
                if let Err(e) = &result {
                    log::error!("autosave of {name}: {e}");
                }
                let _ = with(&store, |s| s.results.push((id, revision, result)));
                ctx.request_repaint();
            });
            Ok(())
        }));
        services.autosave_results = Some(Box::new(move || with(&s2, |s| std::mem::take(&mut s.results)).unwrap_or_default()));
        services.discard_autosave = Some(Box::new(move |id: u64| {
            if let Ok(Some((db, key))) = with(&s3, |s| s.keys.discard(id).and_then(|key| s.db.clone().map(|db| (db, key)))) {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = delete(&db, &key).await {
                        log::warn!("recovery: could not drop {key}: {e}");
                    }
                });
            }
        }));
        services.recover = Some(Box::new(move || {
            with(&s4, |s| std::mem::take(&mut s.entries))
                .unwrap_or_default()
                .into_iter()
                .map(|e| Recoverable {
                    key: e.key,
                    name: e.name,
                    path: e.path,
                    load: Box::new(move || brahma_format::load_from_bytes(&e.bytes).map_err(|e| e.to_string())),
                })
                .collect()
        }));
        services.adopt_autosave = Some(Box::new(move |id: u64, key: &str| {
            let _ = with(&store, |s| s.keys.adopt(id, key));
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_per_launch_and_follow_adoption() {
        let mut a = Keys::new("a");
        let mut b = Keys::new("b");
        assert_ne!(a.key_for(1), b.key_for(1), "two launches must not share a record for document 1");
        assert_eq!(a.key_for(1), a.key_for(1));
        a.adopt(2, "old-7");
        assert_eq!(a.key_for(2), "old-7", "a recovered document keeps writing to the record it came from");
        assert_eq!(a.discard(2), Some("old-7".into()));
        assert_eq!(a.discard(2), None, "discarding twice deletes nothing");
        assert_eq!(a.discard(9), None, "a document that never autosaved has no record");
    }

    #[test]
    fn edits_flush_after_a_pause_and_only_once() {
        let mut idle = IdleFlush::new();
        assert_eq!(idle.tick(0, 0.0), Flush::Idle, "nothing dirty");
        let f1 = fingerprint([(1, 3)].into_iter());
        assert_eq!(idle.tick(f1, 1000.0), Flush::After(5000));
        assert_eq!(idle.tick(f1, 3000.0), Flush::After(3000));
        let f2 = fingerprint([(1, 4)].into_iter());
        assert_ne!(f1, f2);
        assert_eq!(idle.tick(f2, 4000.0), Flush::After(5000), "another edit restarts the pause");
        assert_eq!(idle.tick(f2, 9000.0), Flush::Now);
        assert_eq!(idle.tick(f2, 20000.0), Flush::Idle, "flushed once");
        assert_eq!(idle.tick(0, 21000.0), Flush::Idle, "saved: nothing to flush");
        assert_eq!(idle.tick(f2, 22000.0), Flush::Idle, "the same state again needs no second flush");
    }

    #[test]
    fn fingerprint_is_zero_only_when_clean() {
        assert_eq!(fingerprint(std::iter::empty()), 0);
        assert_ne!(fingerprint([(1, 1)].into_iter()), 0);
        assert_ne!(fingerprint([(1, 1)].into_iter()), fingerprint([(2, 1)].into_iter()));
    }
}
