//! The browser shell: web `Services`, drag-and-drop, and the eframe web runner.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use brahma_codecs::{ChannelLayout, EncodeOptions, Image};
use brahma_doc::Document;
use brahma_ui_egui::i18n;
use brahma_ui_egui::served_fonts;
use brahma_ui_egui::theme::ThemeKind;
use brahma_ui_egui::{BrahmaApp, FileDialogAnswer, FileDialogRequest, Services};
use wasm_bindgen::JsCast as _;
use wasm_bindgen::JsValue;

type Inbox = Arc<Mutex<Vec<(String, Vec<u8>)>>>;

/// Everything File › Open reads: brahma, Photoshop, OpenRaster, Krita and Affinity documents, flat images, and
/// Photoshop brushes (.abr), gradients (.grd) and swatches (.aco, .ase), which go to the preset libraries.
const OPEN_EXTS: &[&str] = &[
    "pcraft", "pdn", "ora", "kra", "krz", "psd", "psb", "psdt", "png", "jpg", "jpeg", "tif", "tiff", "webp", "gif", "bmp", "tga", "ico", "qoi", "exr", "hdr",
    "pbm", "pgm", "ppm", "pam", "pfm", "dds", "fits", "fit", "fts", "dpx", "heic", "heif", "hif", "dng", "cr2", "cr3", "nef", "nrw", "arw", "pef", "orf",
    "rw2", "raf", "abr", "grd", "svg", "svgz", "aco", "ase", "af", "afdesign", "afphoto", "afpub",
];
const SVG_EXTS: &[&str] = &["svg", "svgz"];
const CANVAS_ID: &str = "brahma_canvas";

/// Fonts the host serves next to the page (`brahma_ui_egui::served_fonts`): craft-fonts' manifest
/// format, each file relative to the site root. No manifest (a 404) means no served fonts.
const FONTS_MANIFEST: &str = "fonts/manifest.txt";

pub fn start() {
    eframe::WebLogger::init(log::LevelFilter::Info).ok();
    wasm_bindgen_futures::spawn_local(async {
        let Some(document) = web_sys::window().and_then(|w| w.document()) else {
            log::error!("no document");
            return;
        };
        let Some(canvas) = document.get_element_by_id(CANVAS_ID).and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok()) else {
            log::error!("missing <canvas id=\"{CANVAS_ID}\">");
            return;
        };
        let q = query();
        let force_cpu = q.contains("cpu");
        let mut options = eframe::WebOptions::default();
        brahma_ui_egui::gpu_canvas::use_adapter_limits(&mut options.wgpu_options.wgpu_setup);
        if q.contains("webgl")
            && let eframe::egui_wgpu::WgpuSetup::CreateNew(create) = &mut options.wgpu_options.wgpu_setup
        {
            create.instance_descriptor.backends = eframe::wgpu::Backends::GL;
        }
        // Hydrate before constructing an interactive session: a late load must never undo a
        // user deletion/reorder. Storage denial or timeout still opens a usable session.
        let (database, presets, mut preset_warnings) = match crate::indexed_presets::load().await {
            Ok((db, bridge, warnings)) => (Some(db), bridge, warnings),
            Err(e) => {
                let bridge = crate::preset_bridge::Bridge::default();
                bridge.unavailable(e);
                (None, bridge, Vec::new())
            }
        };
        let pen_target = canvas.clone();
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                options,
                Box::new(move |cc| {
                    BrahmaApp::setup_context(&cc.egui_ctx, ThemeKind::Pro);
                    let inbox: Inbox = Arc::default();
                    load_font_manifest(cc.egui_ctx.clone());
                    let (session, warnings) = crate::preset_bridge::session(presets.clone());
                    preset_warnings.extend(warnings);
                    let mut app = BrahmaApp::new(session, services(inbox.clone()));
                    if !preset_warnings.is_empty() {
                        brahma_ui_egui::notices::post(&mut app, i18n::t("Some brush presets could not be loaded"), preset_warnings, true, None);
                    }
                    listen_pen(&pen_target, app.stylus.feed.clone());
                    app.set_theme(&cc.egui_ctx, ThemeKind::Pro);
                    if let Some(rs) = cc.wgpu_render_state.clone()
                        && !force_cpu
                    {
                        log::info!("brahma-web: wgpu backend {:?}", rs.adapter.get_info().backend);
                        app.set_wgpu(rs);
                    }
                    let unsaved = Arc::new(AtomicBool::new(false));
                    guard_unload(unsaved.clone());
                    let pasted: Pasted = Arc::default();
                    listen_paste(pasted.clone(), cc.egui_ctx.clone());
                    app.services.clipboard_get_image = Some(clipboard_image_service(pasted.clone()));
                    Ok(Box::new(WebShell { app, inbox, title: String::new(), unsaved, pasted, database, presets }))
                }),
            )
            .await;
        if let Some(el) = document.get_element_by_id("brahma_loading") {
            match result {
                Ok(()) => el.remove(),
                Err(e) => el.set_inner_html(&format!("<p>brahma failed to start: {e:?}</p><p>A browser with WebGPU or WebGL2 is required.</p>")),
            }
        }
    });
}

/// Pen pressure, tilt, twist and the eraser button from Pointer Events (eframe forwards none of them for pens) into
/// the app's stylus feed. The sample is kept through `pointerup` so the stroke's last points keep
/// their pressure; hovering, a mouse, or leaving the canvas clears it.
fn listen_pen(target: &web_sys::HtmlCanvasElement, feed: brahma_ui_egui::stylus::StylusFeed) {
    use brahma_ui_egui::stylus::PenSample;
    use wasm_bindgen::closure::Closure;
    for kind in ["pointerdown", "pointermove", "pointerup", "pointercancel", "pointerleave"] {
        let feed = feed.clone();
        let cb = Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |e: web_sys::PointerEvent| {
            let ty = e.type_();
            if ty == "pointerup" && e.pointer_type() == "pen" {
                return;
            }
            let pen = e.pointer_type() == "pen" && e.buttons() != 0 && ty != "pointercancel" && ty != "pointerleave";
            // W3C Pointer Events: `buttons` bit 5 (32) is the pen's eraser.
            let eraser = e.buttons() & 32 != 0;
            feed.set(pen.then(|| PenSample {
                pressure: e.pressure(),
                tilt_x: e.tilt_x() as f32,
                tilt_y: e.tilt_y() as f32,
                rotation: e.twist() as f32,
                eraser,
            }));
        });
        if target.add_event_listener_with_callback(kind, cb.as_ref().unchecked_ref()).is_ok() {
            cb.forget();
        }
    }
}

/// Closing or reloading the tab while a document has unsaved changes asks first, as closing the
/// desktop window does: the browser shows its own "Leave site?" prompt (#1380). `unsaved` is
/// refreshed every frame by [`WebShell`].
/// An image pasted from the OS clipboard: its bytes as the browser handed them over, the
/// decoded RGBA8 copy once `Edit › Paste` asked for it, and whether the paste is still to be
/// applied. Browsers deliver pasted images only through the `paste` DOM event, asynchronously,
/// so the shell keeps the latest one here for `Services::clipboard_get_image`.
#[derive(Default)]
struct PastedImage {
    bytes: Vec<u8>,
    decoded: Option<(u32, u32, Vec<u8>)>,
    pending: bool,
}

type Pasted = Arc<Mutex<PastedImage>>;

/// Catches images pasted with ⌘V / Ctrl+V (a screenshot, a picture copied from another app or
/// page). eframe's own paste handler takes the text and stops the event at the canvas, so this
/// one listens on the window in the capture phase, before it.
fn listen_paste(pasted: Pasted, ctx: egui::Context) {
    use wasm_bindgen::closure::Closure;
    let Some(window) = web_sys::window() else { return };
    let cb = Closure::<dyn FnMut(web_sys::ClipboardEvent)>::new(move |e: web_sys::ClipboardEvent| {
        let Some(data) = e.clipboard_data() else { return };
        let items = data.items();
        let mut file: Option<web_sys::File> = None;
        for i in 0..items.length() {
            if let Some(item) = items.get(i)
                && item.kind() == "file"
                && item.type_().starts_with("image/")
                && let Ok(Some(f)) = item.get_as_file()
            {
                file = Some(f);
                break;
            }
        }
        let Some(file) = file else { return };
        let pasted = pasted.clone();
        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let name = file.name();
            let Ok(buf) = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await else {
                log::error!("couldn't read the pasted image {name}");
                return;
            };
            let bytes = js_sys::Uint8Array::new(&buf).to_vec();
            if bytes.is_empty() {
                return;
            }
            let mut p = pasted.lock().unwrap_or_else(|e| e.into_inner());
            p.bytes = bytes;
            p.decoded = None;
            p.pending = true;
            drop(p);
            ctx.request_repaint();
        });
    });
    if window.add_event_listener_with_callback_and_bool("paste", cb.as_ref().unchecked_ref(), true).is_ok() {
        cb.forget();
    }
}

/// `Services::clipboard_get_image` for the browser: the last pasted image as RGBA8, decoded
/// once (a bad file decodes to nothing and pastes nothing).
fn clipboard_image_service(pasted: Pasted) -> brahma_ui_egui::ClipboardGetFn {
    Box::new(move || {
        let mut p = pasted.lock().unwrap_or_else(|e| e.into_inner());
        if p.bytes.is_empty() {
            return None;
        }
        if p.decoded.is_none() {
            let img = brahma_codecs::decode(&p.bytes).ok()?;
            let rgba = img.convert(ChannelLayout::Rgba, brahma_codecs::SampleType::U8);
            let (w, h) = rgba.dimensions();
            p.decoded = Some((w, h, rgba.data().to_vec()));
        }
        p.decoded.clone()
    })
}

fn guard_unload(unsaved: Arc<AtomicBool>) {
    use wasm_bindgen::closure::Closure;
    let Some(window) = web_sys::window() else { return };
    let cb = Closure::<dyn FnMut(web_sys::BeforeUnloadEvent)>::new(move |e: web_sys::BeforeUnloadEvent| {
        if unsaved.load(Ordering::Relaxed) {
            e.prevent_default();
            // Older browsers show the prompt only when a return value is set.
            e.set_return_value("");
        }
    });
    if window.add_event_listener_with_callback("beforeunload", cb.as_ref().unchecked_ref()).is_ok() {
        cb.forget();
    }
}

fn query() -> String {
    web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default()
}

/// Reads the served font manifest, if the host has one, and lists its families in the font menus.
fn load_font_manifest(ctx: egui::Context) {
    wasm_bindgen_futures::spawn_local(async move {
        // Most hosts serve no fonts: a missing manifest is the normal case, not an error.
        let Ok(bytes) = fetch_bytes(FONTS_MANIFEST, None).await else { return };
        let text = String::from_utf8_lossy(&bytes);
        // A host that answers unknown paths with its HTML page (single-page app fallback).
        if text.trim_start().starts_with('<') {
            return;
        }
        let (fonts, skipped) = served_fonts::parse_manifest(&text);
        for s in skipped {
            log::warn!("{FONTS_MANIFEST}: skipped {s}");
        }
        log::info!("brahma-web: {} served font files", fonts.len());
        // The families for the font menus, their files, and the script fallbacks (the
        // manifest's `scripts`); `served_fonts::serve` fetches them when asked for.
        served_fonts::add_fonts(&fonts);
        // Fetch now the fallbacks of the scripts the browser's languages use (an Arabic reader gets
        // the Arabic font before typing); the rest is fetched when text or a font menu needs it.
        served_fonts::request_for_languages(browser_languages().iter().map(String::as_str));
        ctx.request_repaint();
    });
}

/// The browser's preferred languages (`navigator.languages`, e.g. `["ar-EG", "en"]`).
fn browser_languages() -> Vec<String> {
    web_sys::window().map(|w| w.navigator().languages().iter().filter_map(|l| l.as_string()).collect()).unwrap_or_default()
}

/// GET `url` (relative to the page) and read the whole body. With `integrity` (a Subresource
/// Integrity value), the browser rejects a body whose hash differs.
async fn fetch_bytes(url: &str, integrity: Option<&str>) -> Result<Vec<u8>, String> {
    use wasm_bindgen_futures::JsFuture;
    let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let window = web_sys::window().ok_or("no window")?;
    let request = match integrity {
        Some(sri) => {
            let init = web_sys::RequestInit::new();
            init.set_integrity(sri);
            window.fetch_with_str_and_init(url, &init)
        }
        None => window.fetch_with_str(url),
    };
    let resp: web_sys::Response = JsFuture::from(request).await.map_err(js)?.dyn_into().map_err(|_| "not a Response")?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let body = JsFuture::from(resp.array_buffer().map_err(js)?).await.map_err(js)?;
    Ok(js_sys::Uint8Array::new(&body).to_vec())
}

/// POST `req` and hand the body to `sink` chunk by chunk as the provider streams it.
async fn fetch_stream(req: &brahma_engine::ai_chat::HttpRequest, mut sink: brahma_ui_egui::StreamSink) {
    use brahma_ui_egui::StreamEvent;
    use wasm_bindgen_futures::JsFuture;
    let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let run = async {
        let window = web_sys::window().ok_or("no window")?;
        let headers = web_sys::Headers::new().map_err(js)?;
        for (k, v) in &req.headers {
            headers.set(k, v).map_err(js)?;
        }
        let init = web_sys::RequestInit::new();
        init.set_method("POST");
        init.set_headers(&headers);
        init.set_body(&wasm_bindgen::JsValue::from_str(&req.body));
        let resp: web_sys::Response = JsFuture::from(window.fetch_with_str_and_init(&req.url, &init))
            .await
            .map_err(|e| {
                format!(
                    "the request could not be sent ({}); the provider may not allow calls from a browser page: a desktop build or a proxy endpoint works",
                    js(e)
                )
            })?
            .dyn_into()
            .map_err(|_| "not a Response".to_string())?;
        sink(StreamEvent::Status(resp.status()));
        let body = resp.body().ok_or("no body")?;
        let reader: web_sys::ReadableStreamDefaultReader = body.get_reader().dyn_into().map_err(|_| "no stream reader".to_string())?;
        let mut pending: Vec<u8> = Vec::new();
        loop {
            let chunk = JsFuture::from(reader.read()).await.map_err(js)?;
            let done = js_sys::Reflect::get(&chunk, &"done".into()).ok().and_then(|v| v.as_bool()).unwrap_or(true);
            if done {
                break;
            }
            if let Ok(value) = js_sys::Reflect::get(&chunk, &"value".into()) {
                pending.extend(js_sys::Uint8Array::new(&value).to_vec());
                let valid = match std::str::from_utf8(&pending) {
                    Ok(_) => pending.len(),
                    Err(e) => e.valid_up_to(),
                };
                if valid > 0 {
                    let text = String::from_utf8_lossy(&pending[..valid]).into_owned();
                    pending.drain(..valid);
                    sink(StreamEvent::Chunk(text));
                }
            }
        }
        if !pending.is_empty() {
            sink(StreamEvent::Chunk(String::from_utf8_lossy(&pending).into_owned()));
        }
        Ok::<(), String>(())
    };
    match run.await {
        Ok(()) => sink(StreamEvent::Done),
        Err(e) => sink(StreamEvent::Error(e)),
    }
}

/// POST `req` for the assistant and read the status and body; a provider's error status is an
/// answer, a network or CORS failure is an error.
async fn fetch_post(req: &brahma_engine::ai_chat::HttpRequest) -> Result<(u16, String), String> {
    use wasm_bindgen_futures::JsFuture;
    let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let window = web_sys::window().ok_or("no window")?;
    let headers = web_sys::Headers::new().map_err(js)?;
    for (k, v) in &req.headers {
        headers.set(k, v).map_err(js)?;
    }
    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_headers(&headers);
    init.set_body(&wasm_bindgen::JsValue::from_str(&req.body));
    let resp: web_sys::Response = JsFuture::from(window.fetch_with_str_and_init(&req.url, &init))
        .await
        .map_err(|e| {
            format!(
                "the request could not be sent ({}); the provider may not allow calls from a browser page: a desktop build or a proxy endpoint works",
                js(e)
            )
        })?
        .dyn_into()
        .map_err(|_| "not a Response")?;
    let status = resp.status();
    let text = JsFuture::from(resp.text().map_err(js)?).await.map_err(js)?;
    Ok((status, text.as_string().unwrap_or_default()))
}

/// Replies waiting on the decode worker, by request id.
type DecodeReplies = Rc<RefCell<HashMap<u32, brahma_ui_egui::DecodeReply>>>;

thread_local! {
    static DECODER: RefCell<Option<(web_sys::Worker, DecodeReplies, u32)>> = const { RefCell::new(None) };
}

/// The generated module's JS and wasm URLs, from the preload links trunk writes into the page.
fn module_urls() -> Option<(String, String)> {
    let document = web_sys::window()?.document()?;
    let js = document.query_selector("link[rel=modulepreload]").ok().flatten()?.get_attribute("href")?;
    let wasm = document.query_selector("link[rel=preload][href$='.wasm']").ok().flatten()?.get_attribute("href")?;
    let base = web_sys::Url::new_with_base(&js, &document.base_uri().ok().flatten()?).ok()?;
    let wasm = web_sys::Url::new_with_base(&wasm, &document.base_uri().ok().flatten()?).ok()?;
    Some((base.href(), wasm.href()))
}

/// Start the decode worker (once) and hand it `bytes`; the reply arrives on the page thread.
/// When no worker can be had (an old browser, a blocked script) the reply says so and the
/// page decodes the file inline instead.
fn decode_in_worker(name: String, bytes: Arc<Vec<u8>>, reply: brahma_ui_egui::DecodeReply) {
    use wasm_bindgen::closure::Closure;
    let outcome: Result<(), (String, brahma_ui_egui::DecodeReply)> = DECODER.with(|d| {
        let mut d = d.borrow_mut();
        if d.is_none() {
            let opts = web_sys::WorkerOptions::new();
            opts.set_type(web_sys::WorkerType::Module);
            let Ok(worker) = web_sys::Worker::new_with_options("decode-worker.js", &opts) else {
                return Err(("no decode worker".to_string(), reply));
            };
            let replies: DecodeReplies = Rc::default();
            let waiting = replies.clone();
            let on_message = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
                let data = e.data();
                let id = js_sys::Reflect::get(&data, &"id".into()).ok().and_then(|v| v.as_f64()).map(|v| v as u32);
                let Some(id) = id else { return };
                let Some(reply) = waiting.borrow_mut().remove(&id) else { return };
                let out = js_sys::Reflect::get(&data, &"out".into()).ok().filter(|v| !v.is_undefined() && !v.is_null());
                match out {
                    Some(v) => reply(Ok(js_sys::Uint8Array::new(&v).to_vec())),
                    None => {
                        let err =
                            js_sys::Reflect::get(&data, &"error".into()).ok().and_then(|v| v.as_string()).unwrap_or_else(|| "the decode worker failed".into());
                        reply(Err(err));
                    }
                }
            });
            worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            on_message.forget();
            *d = Some((worker, replies, 0));
        }
        let Some((worker, replies, next)) = d.as_mut() else {
            return Err(("no decode worker".to_string(), reply));
        };
        let Some((js, wasm)) = module_urls() else {
            return Err(("the page's module links are missing".to_string(), reply));
        };
        *next = next.wrapping_add(1);
        let id = *next;
        let msg = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&msg, &"id".into(), &JsValue::from_f64(f64::from(id)));
        let _ = js_sys::Reflect::set(&msg, &"name".into(), &JsValue::from_str(&name));
        let _ = js_sys::Reflect::set(&msg, &"js".into(), &JsValue::from_str(&js));
        let _ = js_sys::Reflect::set(&msg, &"wasm".into(), &JsValue::from_str(&wasm));
        // The bytes are copied into the worker (a transfer would take them from the page, which
        // still needs them for the main-thread half of the import).
        let array = js_sys::Uint8Array::from(bytes.as_slice());
        let _ = js_sys::Reflect::set(&msg, &"bytes".into(), &array.buffer());
        replies.borrow_mut().insert(id, reply);
        if worker.post_message(&msg).is_err()
            && let Some(reply) = replies.borrow_mut().remove(&id)
        {
            return Err(("could not reach the decode worker".to_string(), reply));
        }
        Ok(())
    });
    if let Err((why, reply)) = outcome {
        reply(Err(why));
    }
}

/// Wraps the app to read dropped files asynchronously (browsers can't read them synchronously,
/// so the app's own drop path can't handle them) and feed them through the inbox.
struct WebShell {
    app: BrahmaApp,
    inbox: Inbox,
    /// The browser tab's title as last set: the active document's name (eframe's web backend
    /// ignores `ViewportCommand::Title`, so the shell mirrors it to `document.title`).
    title: String,
    /// Read by the `beforeunload` listener ([`guard_unload`]).
    unsaved: Arc<AtomicBool>,
    /// The last image pasted from the OS clipboard (see [`listen_paste`]).
    pasted: Pasted,
    database: Option<web_sys::IdbDatabase>,
    presets: crate::preset_bridge::Bridge,
}

impl WebShell {
    /// A freshly pasted image goes into the open document as a layer (Edit › Paste), or opens
    /// as a new document when none is open, the moment its bytes arrive.
    fn apply_paste(&mut self) {
        let fresh = {
            let mut p = self.pasted.lock().unwrap_or_else(|e| e.into_inner());
            std::mem::take(&mut p.pending)
        };
        if !fresh {
            return;
        }
        if self.app.session.active().is_some() {
            if let Err(e) = self.app.run("edit.paste", serde_json::json!({})) {
                log::error!("paste: {e}");
            }
        } else {
            let bytes = self.pasted.lock().unwrap_or_else(|e| e.into_inner()).bytes.clone();
            self.inbox.lock().unwrap_or_else(|e| e.into_inner()).push(("pasted image".into(), bytes));
        }
    }

    /// Keep the browser tab (and the installed app's window) titled after the active file.
    fn sync_title(&mut self) {
        let want = brahma_ui_egui::panels::window_title(&self.app);
        if self.title != want
            && let Some(document) = web_sys::window().and_then(|w| w.document())
        {
            document.set_title(&want);
            self.title = want;
        }
    }

    fn save_presets(&mut self, ctx: &egui::Context) {
        if let Some(db) = self.database.clone()
            && let Some(batch) = self.presets.begin()
        {
            let presets = self.presets.clone();
            let wake = ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = crate::indexed_presets::write(&db, &batch).await;
                if let Err(e) = &result {
                    log::error!("Could not save browser presets: {e}");
                }
                presets.finish(batch, result);
                wake.request_repaint();
            });
        }
        self.unsaved.store(self.app.has_unsaved_work() || self.presets.unsaved(), Ordering::Relaxed);
    }
}

impl eframe::App for WebShell {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let dropped = ctx.input_mut(|i| std::mem::take(&mut i.raw.dropped_files));
        for f in dropped {
            let inbox = self.inbox.clone();
            let ctx = ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let name = f.path().file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "dropped".into());
                match f.bytes_async().await {
                    Ok(bytes) => {
                        inbox.lock().unwrap_or_else(|e| e.into_inner()).push((name, bytes));
                        ctx.request_repaint();
                    }
                    Err(e) => log::error!("couldn't read dropped file {name}: {e}"),
                }
            });
        }
        self.apply_paste();
        self.app.logic(ctx, frame);
        self.save_presets(ctx);
        self.sync_title();
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Only while a preset change is unsaved. Storage the browser denies from the start (a
        // sandboxed iframe, blocked site data) loses nothing until the first change, and that
        // change brings the banner with the reason.
        if self.presets.unsaved()
            && let Some(error) = self.presets.error()
        {
            egui::Panel::top("browser_preset_storage").show(ui, |ui| {
                let tokens = brahma_ui_egui::theme::Tokens::for_kind(self.app.ui.theme);
                // Put retry first so floating brush windows cannot cover the control.
                ui.horizontal_wrapped(|ui| {
                    if self.database.is_some() && ui.button(i18n::t("Retry saving presets")).clicked() {
                        self.presets.retry();
                        ui.ctx().request_repaint();
                    }
                    ui.colored_label(tokens.warning, i18n::fmt(i18n::t("Brush preset changes are not saved: {error}"), &[("error", &error)]));
                });
                ui.label(if self.database.is_some() {
                    i18n::t("Keep this tab open to retain your brushes.")
                } else {
                    i18n::t("Browser storage is unavailable; brushes are session-only. Keep this tab open to retain them.")
                });
            });
        }
        self.app.ui(ui, frame);
        self.save_presets(ui.ctx());
    }
}

fn services(inbox: Inbox) -> Services {
    Services {
        screen_pick: screen_color_service(),
        import: Some(Box::new(|name: &str, bytes: &[u8], max_svg_group_depth: usize| {
            brahma_io::import_with_svg_group_depth(name, bytes, max_svg_group_depth).map(|r| (r.document, r.warnings)).map_err(|e| e.to_string())
        })),
        export: Some(Box::new(|doc: &Document, path: &str, settings: &brahma_ui_egui::ExportSettings| {
            brahma_io::export(doc, path, &settings.io_options()).map(|r| (r.bytes, r.warnings)).map_err(|e| e.to_string())
        })),
        file_dialog: Some(Box::new(|request, _parent, reply| match request {
            // The browser's file picker hands over the files' contents, not paths. File › Open
            // asks for several files (`multiple`), each opened in its own tab.
            FileDialogRequest::Open { multiple, extensions, .. } => wasm_bindgen_futures::spawn_local(async move {
                let dialog = if let Some(exts) = extensions {
                    rfd::AsyncFileDialog::new().add_filter("Supported Files", &exts)
                } else {
                    rfd::AsyncFileDialog::new().add_filter("All Formats", OPEN_EXTS).add_filter("SVG", SVG_EXTS)
                };
                let answer = if multiple {
                    match dialog.pick_files().await {
                        Some(picked) if !picked.is_empty() => {
                            let mut files = Vec::with_capacity(picked.len());
                            for file in picked {
                                files.push((file.file_name(), file.read().await));
                            }
                            Some(FileDialogAnswer::ContentsMany(files))
                        }
                        _ => None,
                    }
                } else {
                    match dialog.pick_file().await {
                        Some(file) => Some(FileDialogAnswer::Contents(file.file_name(), file.read().await)),
                        None => None,
                    }
                };
                reply.send(answer);
            }),
            // Browser folder handles cannot be represented as filesystem paths for engine commands.
            FileDialogRequest::Folder { .. } => reply.send(None),
            // No save dialog on the web: the suggested name becomes the download name.
            FileDialogRequest::Save { suggested } => {
                let name = std::path::Path::new(&suggested).file_name().map_or_else(|| suggested.clone(), |n| n.to_string_lossy().to_string());
                reply.send(Some(FileDialogAnswer::SaveTo(name)));
            }
        })),
        write: Some(Box::new(|path: &str, bytes: &[u8]| download(path, bytes))),
        encode_png: Some(Box::new(|w, h, rgba| {
            let img = Image::from_u8(w, h, ChannelLayout::Rgba, rgba.to_vec()).map_err(|e| e.to_string())?;
            brahma_codecs::encode(&img, brahma_codecs::Format::Png, &EncodeOptions::default()).map_err(|e| e.to_string())
        })),
        inbox: Some(inbox),
        decode_async: Some(Box::new(decode_in_worker)),
        fetch_url: Some(Box::new(|url, integrity, reply| {
            wasm_bindgen_futures::spawn_local(async move {
                reply(fetch_bytes(&url, integrity.as_deref()).await);
            });
        })),
        http_stream: Some(Box::new(|req, sink| {
            wasm_bindgen_futures::spawn_local(async move {
                fetch_stream(&req, sink).await;
            });
        })),
        http: Some(Box::new(|req, reply| {
            wasm_bindgen_futures::spawn_local(async move {
                reply(fetch_post(&req).await);
            });
        })),
        // Preferences live in the browser's localStorage.
        load_prefs: Some(Box::new(|| local_storage()?.get_item(PREFS_KEY).ok().flatten())),
        save_prefs: Some(Box::new(|text: &str| local_storage().ok_or("no localStorage")?.set_item(PREFS_KEY, text).map_err(|e| format!("{e:?}")))),
        ..Default::default()
    }
}

const PREFS_KEY: &str = "brahma.preferences";

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

/// Trigger a browser download of `bytes` named after the last component of `path`.
fn download(path: &str, bytes: &[u8]) -> Result<(), String> {
    let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let name = std::path::Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "brahma".into());
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(mime_for(&name));
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &opts).map_err(js)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(js)?;
    let a: web_sys::HtmlAnchorElement = document.create_element("a").map_err(js)?.dyn_into().map_err(|_| "not an anchor")?;
    a.set_href(&url);
    a.set_download(&name);
    a.style().set_property("display", "none").map_err(js)?;
    let body = document.body().ok_or("no body")?;
    body.append_child(&a).map_err(js)?;
    a.click();
    a.remove();
    // Revoke after the click has been dispatched; the download keeps its own reference.
    let revoke = wasm_bindgen::closure::Closure::once_into_js(move || {
        web_sys::Url::revoke_object_url(&url).ok();
    });
    window.set_timeout_with_callback_and_timeout_and_arguments_0(revoke.unchecked_ref(), 10_000).map_err(js)?;
    Ok(())
}

fn mime_for(name: &str) -> &'static str {
    match name.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("tif" | "tiff") => "image/tiff",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("psd" | "psb") => "image/vnd.adobe.photoshop",
        Some("ora") => "image/openraster",
        Some("kra" | "krz") => "application/x-krita",
        _ => "application/octet-stream",
    }
}

/// Rust bindings to the browser's user-activated EyeDropper API; unsupported browsers keep
/// the document picker and its normal canvas zoom. The browser owns its screen magnifier.
fn screen_color_service() -> Option<brahma_ui_egui::screen_picker::Service> {
    use brahma_ui_egui::screen_picker::{Capture, Pending};
    use js_sys::{Function, Reflect};
    use wasm_bindgen::{JsValue, closure::Closure};
    let window = web_sys::window()?;
    let constructor = Reflect::get(&window, &JsValue::from_str("EyeDropper")).ok()?.dyn_into::<Function>().ok()?;
    Some(Box::new(move |ctx| {
        let (tx, rx) = std::sync::mpsc::channel();
        let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let started = (|| -> Result<_, String> {
            let eye = Reflect::construct(&constructor, &js_sys::Array::new()).map_err(|e| format!("Could not start browser eyedropper: {e:?}"))?;
            let controller = web_sys::AbortController::new().map_err(|e| format!("Could not create eyedropper cancellation: {e:?}"))?;
            let options = js_sys::Object::new();
            Reflect::set(&options, &JsValue::from_str("signal"), &controller.signal()).map_err(|e| format!("Could not set eyedropper options: {e:?}"))?;
            let open = Reflect::get(&eye, &JsValue::from_str("open"))
                .map_err(|e| format!("Browser eyedropper has no open method: {e:?}"))?
                .dyn_into::<Function>()
                .map_err(|_| "Invalid browser eyedropper method")?;
            let promise = open
                .call1(&eye, &options)
                .map_err(|e| format!("Could not open browser eyedropper: {e:?}"))?
                .dyn_into::<js_sys::Promise>()
                .map_err(|_| "Browser eyedropper returned no promise")?;
            let stop = cancelled.clone();
            let abort = controller.clone();
            let timer = Closure::<dyn FnMut()>::new(move || {
                if stop.load(std::sync::atomic::Ordering::Relaxed) {
                    abort.abort();
                }
            });
            let interval = window.set_interval_with_callback_and_timeout_and_arguments_0(timer.as_ref().unchecked_ref(), 50).map_err(|e| {
                controller.abort();
                format!("Could not watch eyedropper cancellation: {e:?}")
            })?;
            Ok((promise, timer, interval))
        })();
        match started {
            Err(e) => {
                let _ = tx.send(Err(e));
            }
            Ok((promise, timer, interval)) => {
                let wake = ctx.clone();
                let window = window.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = match wasm_bindgen_futures::JsFuture::from(promise).await {
                        Ok(value) => Reflect::get(&value, &JsValue::from_str("sRGBHex"))
                            .ok()
                            .and_then(|v| v.as_string())
                            .and_then(|h| brahma_ui_egui::color_picker_ui::parse_hex(&h))
                            .map(|rgb| Capture::Color(Some(rgb)))
                            .ok_or_else(|| "Browser eyedropper returned an invalid color".into()),
                        Err(e) if Reflect::get(&e, &JsValue::from_str("name")).ok().and_then(|v| v.as_string()).as_deref() == Some("AbortError") => {
                            Ok(Capture::Color(None))
                        }
                        Err(e) => Err(format!("Browser screen color selection failed: {e:?}")),
                    };
                    window.clear_interval_with_handle(interval);
                    drop(timer);
                    let _ = tx.send(result);
                    wake.request_repaint();
                });
            }
        }
        Pending { receiver: rx, cancelled }
    }))
}
