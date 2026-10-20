//! Window › Assistant: a chat in a side panel that drives the whole app. The model (any
//! provider the person holds a key for, see `brahma_engine::ai::PRESETS`) gets four tools:
//! search the command registry, run any command, inspect the document, look at the canvas
//! as a picture, and generate a picture with the provider's image model (`ai_image`). The shell does the HTTP call through `Services::http` (a browser fetch on the
//! web, a thread on the desktop); the panel polls for the answer each frame, runs the tools the
//! model asked for through the engine, and sends the results back until the model answers in
//! words. Its voice is the persona distilled from the design research (`brahma_aesthetic::PERSONA`).
//!
//! The key never leaves the device except in the request to the provider the person picked.

use std::sync::mpsc::{Receiver, channel};

use brahma_engine::ai::{PRESETS, Protocol};
use std::collections::HashMap;

use brahma_doc::DocId;
use brahma_engine::ai_chat::{self, CANVAS_TOOL, MAX_ROUNDS, Message, Reply, StreamParser, ToolCall};
use brahma_engine::ai_image::{self, IMAGE_TOOL};
use brahma_engine::prefs::AiProvider;
use egui::{Color32, RichText, vec2};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::theme::Tokens;
use crate::{BrahmaApp, StreamEvent, icons, widgets};

/// The long side of the picture the assistant sees.
const CANVAS_PX: u32 = 1024;
const MAX_TOKENS: u32 = 4096;

/// A line of the transcript as shown (the wire-level messages live in `messages`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub kind: LineKind,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineKind {
    User,
    Assistant,
    /// A tool the model ran, in one line.
    Tool,
    Error,
}

/// The answer of one HTTP round: status and body, or a transport error.
pub type HttpOutcome = Result<(u16, String), String>;

/// View state of the assistant panel.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct AssistantUi {
    pub open: bool,
    pub input: String,
    /// Attach a picture of the canvas to every message the person sends.
    pub see_canvas: bool,
    pub width: f32,
    pub show_setup: bool,
    #[serde(skip)]
    pub lines: Vec<Line>,
    #[serde(skip)]
    pub messages: Vec<Message>,
    #[serde(skip)]
    pub busy: bool,
    #[serde(skip)]
    pub rounds: u32,
    #[serde(skip)]
    pub rx: Option<Receiver<HttpOutcome>>,
    /// A streamed answer in flight: its events, the parser assembling them, the HTTP status.
    #[serde(skip)]
    pub stream: Option<Receiver<StreamEvent>>,
    #[serde(skip)]
    pub parser: Option<StreamParser>,
    #[serde(skip)]
    pub stream_status: u16,
    /// Tool calls of the current round still to run (an image generation waits on the network
    /// before the next one runs).
    #[serde(skip)]
    pub pending_calls: Vec<ToolCall>,
    /// A `generate_image` call in flight: the call and where the provider's answer arrives.
    #[serde(skip)]
    pub image_rx: Option<(ToolCall, Receiver<HttpOutcome>)>,
    /// The document this conversation belongs to (`None`: no document), and the others' kept
    /// transcripts: each document has its own conversation.
    #[serde(skip)]
    pub conversation_doc: Option<DocId>,
    #[serde(skip)]
    pub conversations: HashMap<Option<DocId>, (Vec<Line>, Vec<Message>)>,
    /// The preset picked in the setup rows (mirrors the preferences while editing).
    #[serde(skip)]
    pub setup_provider: String,
    #[serde(skip)]
    pub setup_key: String,
    #[serde(skip)]
    pub setup_model: String,
    #[serde(skip)]
    pub setup_image_model: String,
    #[serde(skip)]
    pub setup_base_url: String,
    #[serde(skip)]
    pub setup_loaded: bool,
}

impl Default for AssistantUi {
    fn default() -> Self {
        Self {
            open: false,
            input: String::new(),
            see_canvas: true,
            width: 340.0,
            show_setup: false,
            lines: Vec::new(),
            messages: Vec::new(),
            busy: false,
            rounds: 0,
            rx: None,
            stream: None,
            parser: None,
            stream_status: 0,
            pending_calls: Vec::new(),
            image_rx: None,
            conversation_doc: None,
            conversations: HashMap::new(),
            setup_provider: String::new(),
            setup_key: String::new(),
            setup_model: String::new(),
            setup_image_model: String::new(),
            setup_base_url: String::new(),
            setup_loaded: false,
        }
    }
}

impl Clone for AssistantUi {
    fn clone(&self) -> Self {
        Self {
            open: self.open,
            input: self.input.clone(),
            see_canvas: self.see_canvas,
            width: self.width,
            show_setup: self.show_setup,
            lines: self.lines.clone(),
            messages: self.messages.clone(),
            busy: self.busy,
            rounds: self.rounds,
            rx: None,
            stream: None,
            parser: None,
            stream_status: 0,
            pending_calls: Vec::new(),
            image_rx: None,
            conversation_doc: self.conversation_doc,
            conversations: self.conversations.clone(),
            setup_provider: self.setup_provider.clone(),
            setup_key: self.setup_key.clone(),
            setup_model: self.setup_model.clone(),
            setup_image_model: self.setup_image_model.clone(),
            setup_base_url: self.setup_base_url.clone(),
            setup_loaded: self.setup_loaded,
        }
    }
}

impl std::fmt::Debug for AssistantUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssistantUi").field("open", &self.open).field("lines", &self.lines.len()).field("busy", &self.busy).finish()
    }
}

impl PartialEq for AssistantUi {
    fn eq(&self, o: &Self) -> bool {
        self.open == o.open && self.input == o.input && self.see_canvas == o.see_canvas && self.width == o.width && self.show_setup == o.show_setup
    }
}

/// The canvas as a base64 PNG for the model, at most [`CANVAS_PX`] on the long side.
pub fn canvas_png(app: &BrahmaApp) -> Result<String, String> {
    let d = app.session.active().ok_or("no document is open")?;
    let thumb = brahma_compose::thumbnail(&d.doc, CANVAS_PX);
    let enc = app.services.encode_png.as_ref().ok_or("no PNG encoder configured")?;
    let png = enc(thumb.width, thumb.height, &thumb.pixels)?;
    use base64::Engine as _;
    Ok(base64::engine::general_purpose::STANDARD.encode(png))
}

/// Send what the person typed: attach the canvas when asked, then start the first round.
pub fn send(app: &mut BrahmaApp, ctx: &egui::Context, text: &str) {
    let text = text.trim();
    if text.is_empty() || app.ui.assistant.busy {
        return;
    }
    let mut images = Vec::new();
    if app.ui.assistant.see_canvas && app.session.active().is_some() {
        match canvas_png(app) {
            Ok(png) => images.push(png),
            Err(e) => app.ui.assistant.lines.push(Line { kind: LineKind::Error, text: format!("could not picture the canvas: {e}") }),
        }
    }
    let a = &mut app.ui.assistant;
    a.lines.push(Line { kind: LineKind::User, text: text.to_string() });
    a.messages.push(Message::User { text: text.to_string(), images });
    a.rounds = 0;
    dispatch(app, ctx);
}

/// Build the request for the conversation so far and hand it to the shell's HTTP service.
fn dispatch(app: &mut BrahmaApp, ctx: &egui::Context) {
    let Some(req) = app.session.ai_request() else {
        app.ui.assistant.lines.push(Line { kind: LineKind::Error, text: tl!("Pick a provider and paste your API key in the setup rows above.").to_string() });
        app.ui.assistant.show_setup = true;
        app.ui.assistant.busy = false;
        return;
    };
    let system = ai_chat::system_prompt(&app.session);
    // Streamed where the shell can: the answer appears as it is written.
    if let Some(stream) = app.services.http_stream.as_ref() {
        let request = ai_chat::build_streaming_request(&req, &system, &app.ui.assistant.messages, MAX_TOKENS);
        let (tx, rx) = channel::<StreamEvent>();
        let repaint = ctx.clone();
        stream(
            request,
            Box::new(move |event| {
                let _ = tx.send(event);
                repaint.request_repaint();
            }),
        );
        app.ui.assistant.stream = Some(rx);
        app.ui.assistant.parser = Some(StreamParser::new(req.protocol));
        app.ui.assistant.stream_status = 0;
        app.ui.assistant.busy = true;
        app.ui.assistant.rounds += 1;
        return;
    }
    let Some(http) = app.services.http.as_ref() else {
        app.ui.assistant.lines.push(Line { kind: LineKind::Error, text: tl!("This build has no network access for the assistant.").to_string() });
        app.ui.assistant.busy = false;
        return;
    };
    let request = ai_chat::build_request(&req, &system, &app.ui.assistant.messages, MAX_TOKENS);
    let (tx, rx) = channel::<HttpOutcome>();
    let repaint = ctx.clone();
    http(
        request,
        Box::new(move |outcome| {
            let _ = tx.send(outcome);
            repaint.request_repaint();
        }),
    );
    app.ui.assistant.rx = Some(rx);
    app.ui.assistant.busy = true;
    app.ui.assistant.rounds += 1;
}

/// The turn is over: the conversation belongs to the document now in front (a turn that
/// opened or created a document carries its conversation along with it).
fn finish_turn(app: &mut BrahmaApp) {
    app.ui.assistant.busy = false;
    app.ui.assistant.conversation_doc = app.session.active().map(|d| d.doc.id);
}

/// Each document keeps its own conversation: when the active document changes (and nothing
/// is in flight), the transcript on screen is stashed and the new document's comes back.
fn sync_conversation(app: &mut BrahmaApp) {
    let now = app.session.active().map(|d| d.doc.id);
    if now == app.ui.assistant.conversation_doc || app.ui.assistant.busy {
        return;
    }
    let a = &mut app.ui.assistant;
    let lines = std::mem::take(&mut a.lines);
    let messages = std::mem::take(&mut a.messages);
    if !lines.is_empty() || !messages.is_empty() {
        a.conversations.insert(a.conversation_doc, (lines, messages));
    }
    if let Some((lines, messages)) = a.conversations.remove(&now) {
        a.lines = lines;
        a.messages = messages;
    }
    a.conversation_doc = now;
}

/// Take the streamed events that arrived: feed the parser, finish when the stream ends.
fn poll_stream(app: &mut BrahmaApp, ctx: &egui::Context) -> bool {
    let Some(rx) = app.ui.assistant.stream.as_ref() else { return false };
    let mut finished: Option<Result<(), String>> = None;
    let mut changed = false;
    loop {
        match rx.try_recv() {
            Ok(StreamEvent::Status(s)) => app.ui.assistant.stream_status = s,
            Ok(StreamEvent::Chunk(c)) => {
                if let Some(p) = app.ui.assistant.parser.as_mut() {
                    changed |= p.feed(&c);
                }
            }
            Ok(StreamEvent::Done) => {
                finished = Some(Ok(()));
                break;
            }
            Ok(StreamEvent::Error(e)) => {
                finished = Some(Err(e));
                break;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => break,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                finished = Some(Err("the request was dropped".into()));
                break;
            }
        }
    }
    if changed {
        ctx.request_repaint();
    }
    let Some(outcome) = finished else {
        ctx.request_repaint_after(std::time::Duration::from_millis(120));
        return true;
    };
    app.ui.assistant.stream = None;
    let parser = app.ui.assistant.parser.take();
    let status = app.ui.assistant.stream_status;
    let reply = outcome.and_then(|()| parser.ok_or_else(|| "no parser".to_string())?.finish(status));
    match reply {
        Err(e) => {
            app.ui.assistant.lines.push(Line { kind: LineKind::Error, text: e });
            finish_turn(app);
        }
        Ok(reply) => handle_reply(app, ctx, reply),
    }
    true
}

/// Each frame: take the provider's answer if it arrived, run the tools it asked for, and
/// either send the results back or finish the turn.
pub fn poll(app: &mut BrahmaApp, ctx: &egui::Context) {
    sync_conversation(app);
    if poll_image(app, ctx) || poll_stream(app, ctx) {
        return;
    }
    let Some(rx) = app.ui.assistant.rx.as_ref() else { return };
    let outcome = match rx.try_recv() {
        Ok(o) => o,
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
            return;
        }
        Err(std::sync::mpsc::TryRecvError::Disconnected) => Err("the request was dropped".into()),
    };
    app.ui.assistant.rx = None;
    let protocol = app.session.ai_request().map(|r| r.protocol).unwrap_or(Protocol::OpenAi);
    let reply = outcome.and_then(|(status, body)| ai_chat::parse_response(protocol, status, &body));
    match reply {
        Err(e) => {
            app.ui.assistant.lines.push(Line { kind: LineKind::Error, text: e });
            finish_turn(app);
        }
        Ok(reply) => handle_reply(app, ctx, reply),
    }
}

fn handle_reply(app: &mut BrahmaApp, ctx: &egui::Context, reply: Reply) {
    if !reply.text.trim().is_empty() {
        app.ui.assistant.lines.push(Line { kind: LineKind::Assistant, text: reply.text.trim().to_string() });
    }
    app.ui.assistant.messages.push(Message::Assistant { text: reply.text.clone(), tool_calls: reply.tool_calls.clone() });
    if reply.tool_calls.is_empty() {
        finish_turn(app);
        return;
    }
    app.ui.assistant.pending_calls = reply.tool_calls;
    drain_tools(app, ctx);
}

/// Run the round's remaining tool calls in order. An image generation goes out over the
/// network and the rest wait for it ([`poll_image`] continues here); when every call has its
/// result, the results go back to the model, or the turn ends after too many rounds.
fn drain_tools(app: &mut BrahmaApp, ctx: &egui::Context) {
    while !app.ui.assistant.pending_calls.is_empty() {
        let call = app.ui.assistant.pending_calls.remove(0);
        if call.name == IMAGE_TOOL {
            match start_image(app, ctx, &call) {
                Ok(rx) => {
                    app.ui.assistant.image_rx = Some((call, rx));
                    return;
                }
                Err(e) => {
                    app.ui.assistant.lines.push(Line { kind: LineKind::Tool, text: format!("{}: {e}", tl!("generated a picture")) });
                    app.ui.assistant.messages.push(Message::ToolResult { call_id: call.id.clone(), name: call.name.clone(), content: format!("could not generate the picture: {e}"), images: vec![] });
                }
            }
            continue;
        }
        let result = run_tool(app, &call);
        app.ui.assistant.messages.push(result);
    }
    if app.ui.assistant.rounds >= MAX_ROUNDS {
        app.ui.assistant.lines.push(Line { kind: LineKind::Error, text: tl!("Stopped after too many steps; ask again to continue.").to_string() });
        finish_turn(app);
        return;
    }
    dispatch(app, ctx);
}

/// Send a `generate_image` call to the provider's image model (never through the streaming
/// path: the answer is one JSON body). `Err` is the text the model reads back.
fn start_image(app: &mut BrahmaApp, ctx: &egui::Context, call: &ToolCall) -> Result<Receiver<HttpOutcome>, String> {
    let ask = ai_image::parse_args(&call.args)?;
    let req = app.session.ai_request().ok_or("no provider is set up")?;
    let model = ai_image::image_model(&req, &app.session.prefs().integrations.ai_image_model)?;
    let request = ai_image::build_image_request(&req, &model, &ask)?;
    let http = app.services.http.as_ref().ok_or("this build has no network access for pictures")?;
    let (tx, rx) = channel::<HttpOutcome>();
    let repaint = ctx.clone();
    http(
        request,
        Box::new(move |outcome| {
            let _ = tx.send(outcome);
            repaint.request_repaint();
        }),
    );
    app.ui.assistant.lines.push(Line { kind: LineKind::Tool, text: format!("{} “{}”", tl!("generating a picture of"), ask.name) });
    Ok(rx)
}

/// The provider's picture arrived (or not): place it, tell the model, run the rest of the round.
fn poll_image(app: &mut BrahmaApp, ctx: &egui::Context) -> bool {
    let Some((_, rx)) = app.ui.assistant.image_rx.as_ref() else { return false };
    let outcome = match rx.try_recv() {
        Ok(o) => o,
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
            return true;
        }
        Err(std::sync::mpsc::TryRecvError::Disconnected) => Err("the request was dropped".into()),
    };
    let Some((call, _)) = app.ui.assistant.image_rx.take() else { return false };
    let protocol = app.session.ai_request().map(|r| r.protocol).unwrap_or(Protocol::OpenAi);
    let name = ai_image::parse_args(&call.args).map(|a| a.name).unwrap_or_else(|_| "Generated".into());
    let placed = outcome
        .and_then(|(status, body)| ai_image::parse_image_response(protocol, status, &body))
        .and_then(|bytes| ai_image::place_generated(&mut app.session, &name, bytes).map_err(|e| e.to_string()));
    let (content, trace) = match placed {
        Ok(v) => {
            app.sync_views();
            let where_ = if v["placed"] == "layer" { tl!("as a new layer") } else { tl!("as a new document") };
            (ai_chat::tool_text(&json!({"ok": true, "result": v})), format!("{} “{name}” {where_}", tl!("generated a picture")))
        }
        Err(e) => (format!("could not generate the picture: {e}"), format!("{}: {e}", tl!("generated a picture"))),
    };
    app.ui.assistant.lines.push(Line { kind: LineKind::Tool, text: trace });
    app.ui.assistant.messages.push(Message::ToolResult { call_id: call.id, name: call.name, content, images: vec![] });
    drain_tools(app, ctx);
    true
}

/// Run one tool call: the canvas picture here, everything else in the engine. Every call
/// leaves a one-line trace in the transcript.
fn run_tool(app: &mut BrahmaApp, call: &ToolCall) -> Message {
    let (content, images, trace) = if call.name == CANVAS_TOOL {
        match canvas_png(app) {
            Ok(png) => ("attached: the canvas as it looks now".to_string(), vec![png], tl!("looked at the canvas").to_string()),
            Err(e) => (format!("could not picture the canvas: {e}"), vec![], format!("{}: {e}", tl!("looked at the canvas"))),
        }
    } else {
        // Commands go through the app (prompts, thumbnails and the status line follow), the
        // queries straight to the engine.
        let result = if call.name == "run_command" {
            match call.args.get("id").and_then(Value::as_str) {
                None => json!({"ok": false, "error": "run_command needs `id`"}),
                Some(id) if brahma_engine::commands::find(id).is_none() => {
                    let near: Vec<Value> = ai_chat::search_commands(&app.session, id, 5).into_iter().map(|c| c["id"].clone()).collect();
                    json!({"ok": false, "error": format!("no command `{id}`"), "didYouMean": near})
                }
                Some(id) => match app.run(id, call.args.get("params").cloned().unwrap_or_else(|| json!({}))) {
                    Ok(result) => json!({"ok": true, "result": result}),
                    Err(e) => json!({"ok": false, "error": e}),
                },
            }
        } else {
            match ai_chat::run_tool(&mut app.session, &call.name, &call.args) {
                Ok(v) => v,
                Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        };
        let trace = match call.name.as_str() {
            "run_command" => {
                let id = call.args.get("id").and_then(Value::as_str).unwrap_or("?");
                if result["ok"] == false { format!("{id}: {}", result["error"].as_str().unwrap_or("failed")) } else { format!("{} {id}", tl!("ran")) }
            }
            "search_commands" => format!("{} “{}”", tl!("searched commands for"), call.args.get("query").and_then(Value::as_str).unwrap_or("")),
            "inspect_document" => tl!("read the document").to_string(),
            other => other.to_string(),
        };
        (ai_chat::tool_text(&result), vec![], trace)
    };
    app.ui.assistant.lines.push(Line { kind: LineKind::Tool, text: trace });
    Message::ToolResult { call_id: call.id.clone(), name: call.name.clone(), content, images }
}

/// The options of the provider picker: every preset plus a custom endpoint.
fn provider_options() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> =
        PRESETS.iter().map(|p| (p.id.to_string(), if p.country == "--" { p.name.to_string() } else { format!("{} · {}", p.name, p.country) })).collect();
    v.push(("custom".into(), "custom openai-compatible endpoint".into()));
    v
}

/// Mirror the preferences into the setup rows (once, and after a save).
fn load_setup(app: &mut BrahmaApp) {
    let i = app.session.prefs().integrations.clone();
    let a = &mut app.ui.assistant;
    a.setup_provider = match i.ai_provider {
        AiProvider::Anthropic => "anthropic".into(),
        AiProvider::OpenAi => "openai".into(),
        AiProvider::Google => "google".into(),
        AiProvider::Custom => "custom".into(),
        AiProvider::Preset => i.ai_preset.clone(),
        AiProvider::None => String::new(),
    };
    a.setup_key = match i.ai_provider {
        AiProvider::Anthropic => i.anthropic_api_key.clone(),
        AiProvider::OpenAi => i.openai_api_key.clone(),
        AiProvider::Google => i.google_api_key.clone(),
        AiProvider::Custom => i.custom_api_key.clone(),
        _ => i.preset_api_key.clone(),
    };
    a.setup_model = i.ai_model.clone();
    a.setup_image_model = i.ai_image_model.clone();
    a.setup_base_url = i.custom_base_url.clone();
    a.setup_loaded = true;
}

/// Write the setup rows to the preferences (the provider's own key slot for the three
/// first-party providers, the preset slot for the rest).
fn save_setup(app: &mut BrahmaApp) -> Result<(), String> {
    let a = &app.ui.assistant;
    let mut values = serde_json::Map::new();
    values.insert("integrations.aiModel".into(), json!(a.setup_model.trim()));
    values.insert("integrations.aiImageModel".into(), json!(a.setup_image_model.trim()));
    match a.setup_provider.as_str() {
        "" => {
            values.insert("integrations.aiProvider".into(), json!("none"));
        }
        "anthropic" => {
            values.insert("integrations.aiProvider".into(), json!("anthropic"));
            values.insert("integrations.anthropicApiKey".into(), json!(a.setup_key.trim()));
        }
        "openai" => {
            values.insert("integrations.aiProvider".into(), json!("openai"));
            values.insert("integrations.openaiApiKey".into(), json!(a.setup_key.trim()));
        }
        "google" => {
            values.insert("integrations.aiProvider".into(), json!("google"));
            values.insert("integrations.googleApiKey".into(), json!(a.setup_key.trim()));
        }
        "custom" => {
            values.insert("integrations.aiProvider".into(), json!("custom"));
            values.insert("integrations.customApiKey".into(), json!(a.setup_key.trim()));
            values.insert("integrations.customBaseUrl".into(), json!(a.setup_base_url.trim()));
        }
        preset => {
            values.insert("integrations.aiProvider".into(), json!("preset"));
            values.insert("integrations.aiPreset".into(), json!(preset));
            values.insert("integrations.presetApiKey".into(), json!(a.setup_key.trim()));
        }
    }
    app.run("prefs.set", json!({"values": values})).map(|_| ())
}

/// The side panel, drawn between the canvas and the dock when open.
pub fn panel(app: &mut BrahmaApp, ui: &mut egui::Ui) {
    if !app.ui.assistant.open {
        return;
    }
    if !app.ui.assistant.setup_loaded {
        load_setup(app);
    }
    let t = Tokens::get(ui.ctx());
    let avail = ui.ctx().content_rect().width();
    let max_w = (avail * 0.45).max(220.0);
    let width = app.ui.assistant.width.clamp(220.0, max_w);
    let ready = app.session.ai_request().is_some();
    let mut send_text: Option<String> = None;
    let mut save = false;
    let mut clear = false;
    let resp = crate::direction::end_panel("assistant")
        .resizable(true)
        .default_size(width)
        .min_size(220.0)
        .max_size(max_w)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin::symmetric(10, 8)))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            // Title row.
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                icons::paint(ui, icon_rect, "sparkles", 14.0, t.accent);
                ui.label(RichText::new(tl!("Assistant")).strong().color(t.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icons::button(ui, "x", 20.0, false, tl!("Close")).clicked() {
                        app.ui.assistant.open = false;
                    }
                    if icons::button(ui, "settings", 20.0, app.ui.assistant.show_setup, tl!("Provider and key")).clicked() {
                        app.ui.assistant.show_setup = !app.ui.assistant.show_setup;
                    }
                    if !app.ui.assistant.lines.is_empty() && icons::button(ui, "trash", 20.0, false, tl!("Clear the conversation")).clicked() {
                        clear = true;
                    }
                });
            });
            if !ready && !app.ui.assistant.show_setup {
                app.ui.assistant.show_setup = true;
            }
            if app.ui.assistant.show_setup {
                setup_rows(app, ui, &t, &mut save);
                widgets::hairline(ui);
            }
            // Transcript.
            let input_h = 92.0;
            let list_h = (ui.available_height() - input_h).max(80.0);
            egui::ScrollArea::vertical().id_salt("assistant-transcript").auto_shrink([false, false]).max_height(list_h).stick_to_bottom(true).show(ui, |ui| {
                ui.set_min_height(list_h);
                if app.ui.assistant.lines.is_empty() {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(tl!("Ask for an edit, a look, or advice. The assistant can run every command in brahma and see the canvas."))
                            .color(t.text_dim)
                            .size(12.0),
                    );
                }
                for line in &app.ui.assistant.lines {
                    transcript_line(ui, &t, line);
                }
                // The answer as it is being written.
                if let Some(partial) = app.ui.assistant.parser.as_ref().map(StreamParser::text).filter(|t| !t.trim().is_empty()) {
                    transcript_line(ui, &t, &Line { kind: LineKind::Assistant, text: partial.to_string() });
                }
                if app.ui.assistant.busy {
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new().size(12.0).color(t.accent));
                        ui.label(RichText::new(tl!("thinking…")).color(t.text_faint).size(12.0));
                    });
                }
            });
            widgets::hairline(ui);
            // Input: Enter sends, Shift+Enter breaks the line.
            let id = egui::Id::new("assistant-input");
            let focused = ui.ctx().memory(|m| m.has_focus(id));
            let enter = focused && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            let edit = egui::TextEdit::multiline(&mut app.ui.assistant.input)
                .id(id)
                .desired_rows(2)
                .desired_width(f32::INFINITY)
                .hint_text(tl!("Tell the assistant what to do…"))
                .font(egui::FontId::proportional(12.5));
            ui.add(edit);
            ui.horizontal(|ui| {
                widgets::toggle(ui, &mut app.ui.assistant.see_canvas, tl!("Show it the canvas"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can = !app.ui.assistant.busy && !app.ui.assistant.input.trim().is_empty();
                    ui.add_enabled_ui(can, |ui| {
                        if widgets::primary_button(ui, tl!("Send"), 64.0).clicked() {
                            send_text = Some(std::mem::take(&mut app.ui.assistant.input));
                        }
                    });
                });
            });
            if enter && !app.ui.assistant.busy && !app.ui.assistant.input.trim().is_empty() {
                send_text = Some(std::mem::take(&mut app.ui.assistant.input));
            }
        });
    app.ui.assistant.width = resp.response.rect.width();
    if clear {
        app.ui.assistant.lines.clear();
        app.ui.assistant.messages.clear();
    }
    if save {
        match save_setup(app) {
            Ok(()) => {
                load_setup(app);
                if app.session.ai_request().is_some() {
                    app.ui.assistant.show_setup = false;
                }
            }
            Err(e) => app.ui.assistant.lines.push(Line { kind: LineKind::Error, text: e }),
        }
    }
    if let Some(text) = send_text {
        let ctx = ui.ctx().clone();
        send(app, &ctx, &text);
    }
}

fn setup_rows(app: &mut BrahmaApp, ui: &mut egui::Ui, t: &Tokens, save: &mut bool) {
    ui.label(RichText::new(tl!("Your key stays on this device and goes only to the provider you pick.")).color(t.text_dim).size(11.5));
    let options = provider_options();
    let opts: Vec<(String, &str)> = options.iter().map(|(k, l)| (k.clone(), l.as_str())).collect();
    let w = ui.available_width();
    ui.horizontal(|ui| {
        ui.label(RichText::new(tl!("Provider")).color(t.text_dim));
        widgets::dropdown_literal(ui, "assistant-provider", &mut app.ui.assistant.setup_provider, &opts, (w - 70.0).max(120.0));
    });
    let preset = brahma_engine::ai::preset(&app.ui.assistant.setup_provider);
    let local = preset.is_some_and(|p| p.base_url.starts_with("http://localhost"));
    if app.ui.assistant.setup_provider == "custom" {
        ui.horizontal(|ui| {
            ui.label(RichText::new(tl!("Base URL")).color(t.text_dim));
            widgets::text_field(ui, egui::Id::new("assistant-base-url"), &mut app.ui.assistant.setup_base_url, "https://host/v1", (w - 70.0).max(120.0), true);
        });
    }
    if !local {
        ui.horizontal(|ui| {
            ui.label(RichText::new(tl!("API key")).color(t.text_dim));
            let (rect, _) = ui.allocate_exact_size(vec2((w - 70.0).max(120.0), 24.0), egui::Sense::hover());
            widgets::surface(ui, rect, t.field, false);
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(vec2(6.0, 3.0))));
            child.add(
                egui::TextEdit::singleline(&mut app.ui.assistant.setup_key)
                    .id(egui::Id::new("assistant-key"))
                    .password(true)
                    .frame(egui::Frame::NONE)
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::monospace(12.0)),
            );
        });
    }
    ui.horizontal(|ui| {
        ui.label(RichText::new(tl!("Model")).color(t.text_dim));
        let hint = preset.map(|p| p.default_model).unwrap_or("");
        widgets::text_field(ui, egui::Id::new("assistant-model"), &mut app.ui.assistant.setup_model, hint, (w - 70.0).max(120.0), true);
    });
    ui.horizontal(|ui| {
        ui.label(RichText::new(tl!("Images")).color(t.text_dim)).on_hover_text(tl!("The image model the assistant generates pictures with (blank = the provider's usual one)."));
        let hint = match preset.map(|p| p.protocol) {
            Some(Protocol::Google) => "gemini-2.5-flash-image",
            Some(Protocol::Anthropic) => "",
            _ => "gpt-image-1",
        };
        widgets::text_field(ui, egui::Id::new("assistant-image-model"), &mut app.ui.assistant.setup_image_model, hint, (w - 70.0).max(120.0), true);
    });
    ui.horizontal(|ui| {
        if widgets::primary_button(ui, tl!("Save"), 64.0).clicked() {
            *save = true;
        }
        if app.session.ai_request().is_some() {
            ui.label(RichText::new(tl!("ready")).color(t.accent).size(11.5));
        }
    });
}

fn transcript_line(ui: &mut egui::Ui, t: &Tokens, line: &Line) {
    let w = ui.available_width();
    match line.kind {
        LineKind::User => {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                egui::Frame::NONE.fill(t.accent_soft).corner_radius(t.radius_lg as u8).inner_margin(egui::Margin::symmetric(8, 6)).show(ui, |ui| {
                    ui.set_max_width(w * 0.85);
                    ui.label(RichText::new(&line.text).color(t.text).size(12.5));
                });
            });
        }
        LineKind::Assistant => {
            egui::Frame::NONE
                .fill(t.card)
                .stroke(egui::Stroke::new(1.0, t.card_border))
                .corner_radius(t.radius_lg as u8)
                .inner_margin(egui::Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.set_max_width(w * 0.92);
                    ui.label(RichText::new(&line.text).color(t.text).size(12.5));
                });
        }
        LineKind::Tool => {
            ui.label(RichText::new(format!("· {}", line.text)).color(t.text_faint).size(11.0));
        }
        LineKind::Error => {
            ui.label(RichText::new(&line.text).color(Color32::from_rgb(230, 120, 110)).size(12.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    /// A fake provider: answers each request with the next canned body and records the bodies.
    fn fake_http(answers: Vec<&str>) -> (crate::HttpFn, Arc<Mutex<Vec<String>>>) {
        let queue = Arc::new(Mutex::new(answers.into_iter().map(String::from).collect::<VecDeque<_>>()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let seen2 = seen.clone();
        let f: crate::HttpFn = Box::new(move |req, reply| {
            seen2.lock().unwrap().push(req.body.clone());
            let body = queue.lock().unwrap().pop_front().unwrap_or_else(|| r#"{"error":{"message":"no more answers"}}"#.into());
            reply(Ok((200, body)));
        });
        (f, seen)
    }

    fn harness(answers: Vec<&str>) -> (Harness<'static, BrahmaApp>, Arc<Mutex<Vec<String>>>) {
        let (http, seen) = fake_http(answers);
        let services = crate::Services {
            http: Some(http),
            encode_png: Some(Box::new(|w, h, rgba| {
                let img = brahma_codecs::Image::from_u8(w, h, brahma_codecs::ChannelLayout::Rgba, rgba.to_vec()).map_err(|e| e.to_string())?;
                brahma_codecs::encode(&img, brahma_codecs::Format::Png, &brahma_codecs::EncodeOptions::default()).map_err(|e| e.to_string())
            })),
            ..Default::default()
        };
        let mut app = BrahmaApp::new(brahma_engine::Session::new(), services);
        app.session
            .execute(
                "prefs.set",
                json!({"values": {"integrations.aiProvider": "preset", "integrations.aiPreset": "deepseek", "integrations.presetApiKey": "k"}}),
            )
            .unwrap();
        let mut h = Harness::builder().with_size(vec2(1200.0, 800.0)).build_ui_state(
            |ui, app: &mut BrahmaApp| {
                poll(app, ui.ctx());
                panel(app, ui);
                egui::CentralPanel::default().show(ui, |_| {});
            },
            app,
        );
        BrahmaApp::setup_context(&h.ctx, Default::default());
        h.run_steps(1);
        h.state_mut().ui.assistant.open = true;
        h.run_steps(2);
        (h, seen)
    }

    #[test]
    fn the_assistant_runs_the_commands_the_model_asks_for_and_shows_the_answer() {
        let (mut h, seen) = harness(vec![
            r##"{"choices":[{"message":{"content":null,"tool_calls":[{"id":"c1","type":"function","function":{"name":"run_command","arguments":"{\"id\":\"file.new\",\"params\":{\"width\":40,\"height\":30,\"background\":\"#336699\"}}"}},{"id":"c2","type":"function","function":{"name":"look_at_canvas","arguments":"{}"}}]},"finish_reason":"tool_calls"}]}"##,
            r#"{"choices":[{"message":{"content":"made a 40 by 30 blue canvas."},"finish_reason":"stop"}]}"#,
        ]);
        h.state_mut().ui.assistant.input = "new blue canvas".into();
        h.run_steps(1);
        h.get_by_label("Send").click();
        h.run_steps(4);
        let app = h.state();
        assert_eq!(app.session.active().map(|d| (d.doc.size.width, d.doc.size.height)), Some((40, 30)));
        assert!(!app.ui.assistant.busy);
        let kinds: Vec<LineKind> = app.ui.assistant.lines.iter().map(|l| l.kind).collect();
        assert_eq!(kinds, vec![LineKind::User, LineKind::Tool, LineKind::Tool, LineKind::Assistant], "{:?}", app.ui.assistant.lines);
        assert_eq!(app.ui.assistant.lines[3].text, "made a 40 by 30 blue canvas.");
        // The second request carried the tool results and the canvas picture.
        let bodies = seen.lock().unwrap();
        assert_eq!(bodies.len(), 2);
        let second: Value = serde_json::from_str(&bodies[1]).unwrap();
        let msgs = second["messages"].as_array().unwrap();
        assert!(msgs.iter().any(|m| m["role"] == "tool" && m["tool_call_id"] == "c1"));
        assert!(msgs.iter().any(|m| m["content"].as_array().is_some_and(|c| c.iter().any(|p| p["type"] == "image_url"))), "the canvas picture");
        assert!(second["messages"][0]["content"].as_str().unwrap().contains("shepherd"), "the persona is the system prompt");
        assert!(!bodies[0].contains("\"k\""), "the key goes in headers, never in the body");
    }

    /// The model asks for a picture: the image request goes to the provider's image endpoint,
    /// the bytes land as a layer named after the prompt, and the model hears where it went.
    #[test]
    fn a_generated_picture_lands_as_a_layer_and_the_model_hears_about_it() {
        use base64::Engine as _;
        let img = brahma_codecs::Image::from_u8(6, 4, brahma_codecs::ChannelLayout::Rgba, vec![90; 6 * 4 * 4]).unwrap();
        let png = brahma_codecs::encode(&img, brahma_codecs::Format::Png, &brahma_codecs::EncodeOptions::default()).unwrap();
        let image_body = json!({"data": [{"b64_json": base64::engine::general_purpose::STANDARD.encode(png)}]}).to_string();
        let (mut h, seen) = harness(vec![
            r##"{"choices":[{"message":{"content":null,"tool_calls":[{"id":"c1","type":"function","function":{"name":"generate_image","arguments":"{\"prompt\":\"a red fox at dusk\",\"size\":\"1024x1024\"}"}}]},"finish_reason":"tool_calls"}]}"##,
            &image_body,
            r#"{"choices":[{"message":{"content":"placed the fox."},"finish_reason":"stop"}]}"#,
        ]);
        h.state_mut().run("file.new", json!({"width": 40, "height": 30})).unwrap();
        h.state_mut().ui.assistant.input = "add a fox".into();
        h.run_steps(1);
        h.get_by_label("Send").click();
        h.run_steps(6);
        let app = h.state();
        assert!(!app.ui.assistant.busy, "{:?}", app.ui.assistant.lines);
        let doc = &app.session.active().unwrap().doc;
        assert_eq!(doc.layers.len(), 2, "the picture is a new layer: {:?} / {:?}", app.ui.assistant.lines, seen.lock().unwrap());
        assert_eq!(doc.layers.last().unwrap().name, "a red fox at dusk");
        let bodies = seen.lock().unwrap();
        assert_eq!(bodies.len(), 3);
        let image_req: Value = serde_json::from_str(&bodies[1]).unwrap();
        assert_eq!(image_req["prompt"], "a red fox at dusk");
        assert_eq!(image_req["model"], "gpt-image-1", "an OpenAI-compatible preset asks its usual image model");
        let third: Value = serde_json::from_str(&bodies[2]).unwrap();
        let tool_msg = third["messages"].as_array().unwrap().iter().find(|m| m["role"] == "tool" && m["tool_call_id"] == "c1").unwrap();
        assert!(tool_msg["content"].as_str().unwrap().contains("\"placed\":\"layer\""), "{tool_msg}");
        assert_eq!(app.ui.assistant.lines.last().unwrap().text, "placed the fox.");
        assert!(app.ui.assistant.lines.iter().any(|l| l.kind == LineKind::Tool && l.text.contains("as a new layer")), "{:?}", app.ui.assistant.lines);
    }

    /// A streaming shell: the text shows as it arrives, the tool call runs when the stream
    /// ends, and each document keeps its own conversation.
    #[test]
    fn streamed_answers_appear_as_they_arrive_and_conversations_follow_the_document() {
        let bodies: Arc<Mutex<Vec<String>>> = Arc::default();
        let seen = bodies.clone();
        let answers = Arc::new(Mutex::new(VecDeque::from(vec![
            vec!["data: {\"choices\":[{\"delta\":{\"content\":\"making \"}}]}\n\n".to_string(), "data: {\"choices\":[{\"delta\":{\"content\":\"it\"}}]}\n\n".to_string(), "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"c1\",\"function\":{\"name\":\"run_command\",\"arguments\":\"{\\\"id\\\":\\\"file.new\\\",\\\"params\\\":{\\\"width\\\":20,\\\"height\\\":10}}\"}}]},\"finish_reason\":\"tool_calls\"}]}\n\ndata: [DONE]\n\n".to_string()],
            vec!["data: {\"choices\":[{\"delta\":{\"content\":\"done.\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n".to_string()],
        ])));
        let stream: crate::HttpStreamFn = Box::new(move |req, mut sink| {
            seen.lock().unwrap().push(req.body.clone());
            assert!(req.body.contains("\"stream\":true"));
            sink(StreamEvent::Status(200));
            for chunk in answers.lock().unwrap().pop_front().unwrap_or_default() {
                sink(StreamEvent::Chunk(chunk));
            }
            sink(StreamEvent::Done);
        });
        let services = crate::Services { http_stream: Some(stream), ..Default::default() };
        let mut app = BrahmaApp::new(brahma_engine::Session::new(), services);
        app.session
            .execute(
                "prefs.set",
                json!({"values": {"integrations.aiProvider": "preset", "integrations.aiPreset": "deepseek", "integrations.presetApiKey": "k"}}),
            )
            .unwrap();
        let ctx = egui::Context::default();
        send(&mut app, &ctx, "new small canvas");
        assert!(app.ui.assistant.busy && app.ui.assistant.stream.is_some());
        poll(&mut app, &ctx);
        // The first stream ended with a tool call: the command ran and the second request went out.
        assert_eq!(app.session.active().map(|d| (d.doc.size.width, d.doc.size.height)), Some((20, 10)));
        poll(&mut app, &ctx);
        assert!(!app.ui.assistant.busy);
        let texts: Vec<&str> = app.ui.assistant.lines.iter().filter(|l| l.kind == LineKind::Assistant).map(|l| l.text.as_str()).collect();
        assert_eq!(texts, vec!["making it", "done."]);
        assert_eq!(bodies.lock().unwrap().len(), 2);
        // Another document: a fresh conversation; back again: the old one.
        let lines_before = app.ui.assistant.lines.len();
        app.run("file.new", json!({"width": 8, "height": 8})).unwrap();
        poll(&mut app, &ctx);
        assert!(app.ui.assistant.lines.is_empty(), "the new document starts clean");
        app.session.set_active(0);
        poll(&mut app, &ctx);
        assert_eq!(app.ui.assistant.lines.len(), lines_before, "the first document's conversation is back");
    }

    #[test]
    fn provider_errors_show_without_a_crash_and_the_setup_rows_save_the_key() {
        let (mut h, _) = harness(vec![r#"{"error":{"message":"invalid api key"}}"#]);
        h.state_mut().ui.assistant.input = "hello".into();
        h.run_steps(1);
        h.get_by_label("Send").click();
        h.run_steps(3);
        let lines = &h.state().ui.assistant.lines;
        assert!(lines.iter().any(|l| l.kind == LineKind::Error && l.text.contains("invalid api key")), "{lines:?}");
        assert!(!h.state().ui.assistant.busy);
        // Setup: switch to mistral with a new key and save.
        h.state_mut().ui.assistant.show_setup = true;
        h.state_mut().ui.assistant.setup_provider = "mistral".into();
        h.state_mut().ui.assistant.setup_key = "m-key".into();
        h.state_mut().ui.assistant.setup_model = String::new();
        h.run_steps(1);
        h.get_by_label("Save").click();
        h.run_steps(2);
        let i = h.state().session.prefs().integrations.clone();
        assert_eq!((i.ai_provider, i.ai_preset.as_str(), i.preset_api_key.as_str()), (AiProvider::Preset, "mistral", "m-key"));
        assert_eq!(h.state().session.ai_request().unwrap().base_url, "https://api.mistral.ai/v1");
        assert!(!h.state().ui.assistant.show_setup, "ready: the setup folds away");
        // No provider at all: sending explains instead of failing silently.
        h.state_mut().session.execute("prefs.set", json!({"values": {"integrations.aiProvider": "none"}})).unwrap();
        h.state_mut().ui.assistant.input = "hi".into();
        h.run_steps(1);
        h.get_by_label("Send").click();
        h.run_steps(2);
        assert!(h.state().ui.assistant.lines.last().is_some_and(|l| l.kind == LineKind::Error));
        assert!(h.state().ui.assistant.show_setup);
        assert!(h.query_by_label("Provider").is_some());
    }
}
