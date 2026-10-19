//! The assistant's conversation layer: one neutral message shape, serialised to each provider's
//! wire protocol (Anthropic messages, OpenAI chat completions and everything compatible with
//! it, Google Gemini) and parsed back; the four tools the model drives brahma with; and the
//! system prompt that carries the persona.
//!
//! No I/O here: the shell performs the HTTP call (a browser `fetch` on the web, a thread on the
//! desktop) and hands the body back to [`parse_response`]. The engine executes the tools, so the
//! assistant can do anything a command can, through [`run_tool`]. `look_at_canvas` is the one
//! tool the shell answers itself, with a PNG of the document.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::ai::{AiRequest, Protocol};
use crate::{EngineError, Result, Session};

/// A tool the model asked for: the provider's call id (made up for providers without one),
/// the tool name and its arguments.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: Value,
}

/// One turn of the conversation. Images are base64 PNGs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "camelCase")]
pub enum Message {
    User {
        text: String,
        #[serde(default)]
        images: Vec<String>,
    },
    Assistant {
        text: String,
        #[serde(default)]
        tool_calls: Vec<ToolCall>,
    },
    /// What a tool answered; `images` for `look_at_canvas`.
    ToolResult {
        call_id: String,
        name: String,
        content: String,
        #[serde(default)]
        images: Vec<String>,
    },
}

/// A request ready to send: POST `url` with `headers` and the JSON `body`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

/// What the model answered: text, the tools it wants run, and why it stopped.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reply {
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
    pub stop: String,
}

/// The tool that the shell answers with a picture of the document.
pub const CANVAS_TOOL: &str = "look_at_canvas";
/// Tool results longer than this are cut (the model gets the start and a note).
const MAX_TOOL_TEXT: usize = 24_000;
/// The most tool-call rounds one user message may trigger before the assistant has to answer.
pub const MAX_ROUNDS: u32 = 16;

/// The tools, as (name, description, JSON schema of the arguments).
pub fn tools() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "search_commands",
            "find brahma commands by words in their id, label or menu path. returns ids with their parameter documentation. use it before run_command when unsure of an id or its params.",
            json!({"type": "object", "properties": {"query": {"type": "string", "description": "words to look for, e.g. 'gaussian blur' or 'new layer'"}, "limit": {"type": "integer", "description": "at most this many results (default 12)"}}, "required": ["query"]}),
        ),
        (
            "run_command",
            "run a brahma command by id with JSON params; every edit, file, layer, selection, filter, adjustment, text, paint and export action is a command. the result says ok or the error.",
            json!({"type": "object", "properties": {"id": {"type": "string", "description": "the command id, e.g. 'layer.new'"}, "params": {"type": "object", "description": "the command's parameters as documented by search_commands"}}, "required": ["id"]}),
        ),
        (
            "inspect_document",
            "the open documents and the active one in detail: size, mode, depth, layers (ids, names, kinds, visibility, opacity, blend), selection bounds, history, foreground and background colours.",
            json!({"type": "object", "properties": {}}),
        ),
        (
            CANVAS_TOOL,
            "see the active document as it looks now (a PNG of the composite, at most 1024 px on the long side). call it to judge the image, to find what to change, or to verify an edit.",
            json!({"type": "object", "properties": {}}),
        ),
    ]
}

/// The system prompt: the persona plus the facts the model needs about this session.
pub fn system_prompt(s: &Session) -> String {
    let mut p = String::from(brahma_aesthetic::PERSONA);
    p.push_str("\n\nthis session:\n");
    p.push_str(&format!("- brahma {}; commands available: {}\n", crate::build_info::VERSION, crate::commands::command_specs().len()));
    match s.active() {
        Some(d) => {
            p.push_str(&format!(
                "- active document: \"{}\", {} x {} px, {:?}, {} bits, {} layers{}\n",
                d.doc.name,
                d.doc.size.width,
                d.doc.size.height,
                d.doc.mode,
                d.doc.depth.bits(),
                d.doc.layers.len(),
                if d.doc.selection.is_some() { ", with a selection" } else { "" }
            ));
        }
        None => p.push_str("- no document is open: create one with file.new {width, height, background} or ask which file to open\n"),
    }
    p.push_str("- the person can see the canvas; after an edit say what changed, not how commands work\n");
    p
}

/// `messages` as the provider's request body, ready to POST.
pub fn build_request(req: &AiRequest, system: &str, messages: &[Message], max_tokens: u32) -> HttpRequest {
    let mut headers = req.headers.clone();
    headers.push(("Content-Type".into(), "application/json".into()));
    match req.protocol {
        Protocol::Anthropic => {
            HttpRequest { url: format!("{}/messages", req.base_url), headers, body: anthropic_body(req, system, messages, max_tokens).to_string() }
        }
        Protocol::OpenAi => {
            HttpRequest { url: format!("{}/chat/completions", req.base_url), headers, body: openai_body(req, system, messages, max_tokens).to_string() }
        }
        Protocol::Google => HttpRequest {
            url: format!("{}/models/{}:generateContent", req.base_url, req.model),
            headers,
            body: google_body(system, messages, max_tokens).to_string(),
        },
    }
}

fn anthropic_body(req: &AiRequest, system: &str, messages: &[Message], max_tokens: u32) -> Value {
    let tools: Vec<Value> = tools().into_iter().map(|(n, d, s)| json!({"name": n, "description": d, "input_schema": s})).collect();
    let mut out: Vec<Value> = Vec::new();
    for m in messages {
        match m {
            Message::User { text, images } => {
                let mut content = Vec::new();
                for i in images {
                    content.push(json!({"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": i}}));
                }
                content.push(json!({"type": "text", "text": if text.is_empty() { " " } else { text }}));
                out.push(json!({"role": "user", "content": content}));
            }
            Message::Assistant { text, tool_calls } => {
                let mut content = Vec::new();
                if !text.trim().is_empty() {
                    content.push(json!({"type": "text", "text": text}));
                }
                for c in tool_calls {
                    content.push(json!({"type": "tool_use", "id": c.id, "name": c.name, "input": if c.args.is_object() { c.args.clone() } else { json!({}) }}));
                }
                if content.is_empty() {
                    content.push(json!({"type": "text", "text": " "}));
                }
                out.push(json!({"role": "assistant", "content": content}));
            }
            Message::ToolResult { call_id, content, images, .. } => {
                let mut inner = vec![json!({"type": "text", "text": if content.is_empty() { "ok" } else { content }})];
                for i in images {
                    inner.push(json!({"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": i}}));
                }
                let block = json!({"type": "tool_result", "tool_use_id": call_id, "content": inner});
                // Every result of one assistant turn goes into a single user message.
                match out.last_mut() {
                    Some(last) if last["role"] == "user" && last["content"][0]["type"] == "tool_result" => {
                        if let Some(arr) = last["content"].as_array_mut() {
                            arr.push(block);
                        }
                    }
                    _ => out.push(json!({"role": "user", "content": [block]})),
                }
            }
        }
    }
    json!({"model": req.model, "max_tokens": max_tokens, "system": system, "tools": tools, "messages": out})
}

fn data_url(png_base64: &str) -> String {
    format!("data:image/png;base64,{png_base64}")
}

fn openai_body(req: &AiRequest, system: &str, messages: &[Message], max_tokens: u32) -> Value {
    let tools: Vec<Value> =
        tools().into_iter().map(|(n, d, s)| json!({"type": "function", "function": {"name": n, "description": d, "parameters": s}})).collect();
    let mut out: Vec<Value> = vec![json!({"role": "system", "content": system})];
    for m in messages {
        match m {
            Message::User { text, images } => {
                if images.is_empty() {
                    out.push(json!({"role": "user", "content": text}));
                } else {
                    let mut content = vec![json!({"type": "text", "text": text})];
                    for i in images {
                        content.push(json!({"type": "image_url", "image_url": {"url": data_url(i)}}));
                    }
                    out.push(json!({"role": "user", "content": content}));
                }
            }
            Message::Assistant { text, tool_calls } => {
                let mut msg = json!({"role": "assistant", "content": if text.is_empty() { Value::Null } else { json!(text) }});
                if !tool_calls.is_empty() {
                    let calls: Vec<Value> = tool_calls
                        .iter()
                        .map(|c| json!({"id": c.id, "type": "function", "function": {"name": c.name, "arguments": c.args.to_string()}}))
                        .collect();
                    msg["tool_calls"] = json!(calls);
                }
                out.push(msg);
            }
            Message::ToolResult { call_id, content, images, .. } => {
                out.push(json!({"role": "tool", "tool_call_id": call_id, "content": if content.is_empty() { "ok" } else { content }}));
                if !images.is_empty() {
                    // Tool messages are text-only here: the picture follows as a user message.
                    let mut c = vec![json!({"type": "text", "text": "the canvas now:"})];
                    for i in images {
                        c.push(json!({"type": "image_url", "image_url": {"url": data_url(i)}}));
                    }
                    out.push(json!({"role": "user", "content": c}));
                }
            }
        }
    }
    json!({"model": req.model, "max_tokens": max_tokens, "tools": tools, "messages": out})
}

fn google_body(system: &str, messages: &[Message], max_tokens: u32) -> Value {
    let decls: Vec<Value> = tools().into_iter().map(|(n, d, s)| json!({"name": n, "description": d, "parameters": s})).collect();
    let mut contents: Vec<Value> = Vec::new();
    for m in messages {
        match m {
            Message::User { text, images } => {
                let mut parts = vec![json!({"text": if text.is_empty() { " " } else { text }})];
                for i in images {
                    parts.push(json!({"inline_data": {"mime_type": "image/png", "data": i}}));
                }
                contents.push(json!({"role": "user", "parts": parts}));
            }
            Message::Assistant { text, tool_calls } => {
                let mut parts = Vec::new();
                if !text.trim().is_empty() {
                    parts.push(json!({"text": text}));
                }
                for c in tool_calls {
                    parts.push(json!({"functionCall": {"name": c.name, "args": if c.args.is_object() { c.args.clone() } else { json!({}) }}}));
                }
                if parts.is_empty() {
                    parts.push(json!({"text": " "}));
                }
                contents.push(json!({"role": "model", "parts": parts}));
            }
            Message::ToolResult { name, content, images, .. } => {
                let mut parts = vec![json!({"functionResponse": {"name": name, "response": {"content": if content.is_empty() { "ok" } else { content }}}})];
                for i in images {
                    parts.push(json!({"inline_data": {"mime_type": "image/png", "data": i}}));
                }
                match contents.last_mut() {
                    Some(last) if last["role"] == "user" && last["parts"][0].get("functionResponse").is_some() => {
                        if let Some(arr) = last["parts"].as_array_mut() {
                            arr.extend(parts);
                        }
                    }
                    _ => contents.push(json!({"role": "user", "parts": parts})),
                }
            }
        }
    }
    json!({
        "system_instruction": {"parts": [{"text": system}]},
        "contents": contents,
        "tools": [{"function_declarations": decls}],
        "generationConfig": {"maxOutputTokens": max_tokens},
    })
}

/// The provider's answer, or its error message. `status` is the HTTP status (0 = unknown).
pub fn parse_response(protocol: Protocol, status: u16, body: &str) -> std::result::Result<Reply, String> {
    let v: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) if status >= 400 => return Err(format!("the provider answered HTTP {status}: {}", snippet(body))),
        Err(e) => return Err(format!("the provider's answer is not JSON ({e}): {}", snippet(body))),
    };
    if let Some(msg) = provider_error(&v) {
        return Err(if status >= 400 { format!("HTTP {status}: {msg}") } else { msg });
    }
    if status >= 400 {
        return Err(format!("the provider answered HTTP {status}: {}", snippet(body)));
    }
    let mut reply = Reply::default();
    match protocol {
        Protocol::Anthropic => {
            for block in v["content"].as_array().into_iter().flatten() {
                match block["type"].as_str() {
                    Some("text") => push_text(&mut reply.text, block["text"].as_str().unwrap_or_default()),
                    Some("tool_use") => reply.tool_calls.push(ToolCall {
                        id: block["id"].as_str().unwrap_or("call").to_string(),
                        name: block["name"].as_str().unwrap_or_default().to_string(),
                        args: block["input"].clone(),
                    }),
                    _ => {}
                }
            }
            reply.stop = v["stop_reason"].as_str().unwrap_or_default().to_string();
        }
        Protocol::OpenAi => {
            let msg = &v["choices"][0]["message"];
            match &msg["content"] {
                Value::String(s) => push_text(&mut reply.text, s),
                Value::Array(parts) => {
                    for p in parts {
                        if let Some(t) = p["text"].as_str() {
                            push_text(&mut reply.text, t);
                        }
                    }
                }
                _ => {}
            }
            for (i, c) in msg["tool_calls"].as_array().into_iter().flatten().enumerate() {
                let raw = c["function"]["arguments"].as_str().unwrap_or("{}");
                let args = serde_json::from_str::<Value>(raw).unwrap_or_else(|_| json!({"_raw": raw}));
                reply.tool_calls.push(ToolCall {
                    id: c["id"].as_str().map(str::to_string).unwrap_or_else(|| format!("call-{i}")),
                    name: c["function"]["name"].as_str().unwrap_or_default().to_string(),
                    args,
                });
            }
            reply.stop = v["choices"][0]["finish_reason"].as_str().unwrap_or_default().to_string();
        }
        Protocol::Google => {
            let cand = &v["candidates"][0];
            for (i, part) in cand["content"]["parts"].as_array().into_iter().flatten().enumerate() {
                if let Some(t) = part["text"].as_str() {
                    push_text(&mut reply.text, t);
                }
                if let Some(fc) = part.get("functionCall") {
                    reply.tool_calls.push(ToolCall {
                        id: format!("call-{i}"),
                        name: fc["name"].as_str().unwrap_or_default().to_string(),
                        args: fc["args"].clone(),
                    });
                }
            }
            reply.stop = cand["finishReason"].as_str().unwrap_or_default().to_string();
            if reply.text.is_empty()
                && reply.tool_calls.is_empty()
                && let Some(reason) = v["promptFeedback"]["blockReason"].as_str()
            {
                return Err(format!("the provider blocked the request: {reason}"));
            }
        }
    }
    Ok(reply)
}

fn push_text(into: &mut String, t: &str) {
    if t.is_empty() {
        return;
    }
    if !into.is_empty() {
        into.push('\n');
    }
    into.push_str(t);
}

fn provider_error(v: &Value) -> Option<String> {
    let e = v.get("error")?;
    Some(match e {
        Value::String(s) => s.clone(),
        Value::Object(o) => o.get("message").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| e.to_string()),
        other => other.to_string(),
    })
}

fn snippet(body: &str) -> String {
    let t = body.trim();
    let cut: String = t.chars().take(300).collect();
    if cut.len() < t.len() { format!("{cut}…") } else { cut }
}

/// Run one of the engine-side tools. `look_at_canvas` is the shell's; asking the engine for it
/// is an error, so a shell that forgot to answer it fails loudly rather than silently.
pub fn run_tool(s: &mut Session, name: &str, args: &Value) -> Result<Value> {
    match name {
        "search_commands" => {
            let query = args.get("query").and_then(Value::as_str).unwrap_or_default();
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(12).clamp(1, 50) as usize;
            Ok(json!({"commands": search_commands(s, query, limit)}))
        }
        "run_command" => {
            let id = args.get("id").and_then(Value::as_str).ok_or_else(|| EngineError::Other("run_command needs `id`".into()))?;
            let params = args.get("params").cloned().unwrap_or_else(|| json!({}));
            if crate::commands::find(id).is_none() {
                let near = search_commands(s, id, 5).into_iter().filter_map(|c| c["id"].as_str().map(str::to_string)).collect::<Vec<_>>();
                return Ok(json!({"ok": false, "error": format!("no command `{id}`"), "didYouMean": near}));
            }
            Ok(match s.execute(id, params) {
                Ok(result) => json!({"ok": true, "result": result}),
                Err(e) => json!({"ok": false, "error": e.to_string()}),
            })
        }
        "inspect_document" => Ok(json!({"session": crate::inspect::session(s), "document": s.active().map(crate::inspect::document)})),
        CANVAS_TOOL => Err(EngineError::Other("look_at_canvas is answered by the app shell".into())),
        other => Ok(json!({"ok": false, "error": format!("unknown tool `{other}`"), "tools": tools().iter().map(|t| t.0).collect::<Vec<_>>()})),
    }
}

/// The best `limit` commands for `query`: every query word must appear in the id, label or
/// menu path; exact id matches first, then shorter ids.
pub fn search_commands(s: &Session, query: &str, limit: usize) -> Vec<Value> {
    let words: Vec<String> = query.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_lowercase).collect();
    let mut hits: Vec<(u32, &crate::commands::CommandSpec)> = crate::commands::command_specs()
        .iter()
        .filter_map(|c| {
            let hay = format!("{} {} {}", c.id, c.label, c.menu.join(" ")).to_lowercase();
            let id = c.id.to_lowercase();
            if words.is_empty() || words.iter().all(|w| hay.contains(w.as_str())) {
                let score = if id == query.to_lowercase() {
                    0
                } else if words.iter().all(|w| id.contains(w.as_str())) {
                    1
                } else {
                    2
                };
                Some((score * 1000 + c.id.len() as u32, c))
            } else {
                None
            }
        })
        .collect();
    hits.sort_by_key(|(k, c)| (*k, c.id));
    hits.into_iter()
        .take(limit)
        .map(|(_, c)| json!({"id": c.id, "label": c.label, "menu": c.menu, "params": c.params, "enabled": s.is_enabled(c.id)}))
        .collect()
}

/// A tool result as the text the model reads, cut to a sane size.
pub fn tool_text(v: &Value) -> String {
    let s = match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    if s.len() <= MAX_TOOL_TEXT {
        return s;
    }
    let mut end = MAX_TOOL_TEXT;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… [cut: {} more characters]", &s[..end], s.len() - end)
}

/// `ai.tool`: run an assistant tool from the CLI, the control channel or MCP, so an outside
/// agent can use the same search and inspection the built-in assistant has.
fn ai_tool(s: &mut Session, p: &Value) -> Result<Value> {
    let name =
        p.get("name").and_then(Value::as_str).ok_or_else(|| EngineError::BadParams { cmd: "ai.tool".into(), msg: "`name` (a string) is required".into() })?;
    let args = p.get("args").cloned().unwrap_or_else(|| Value::Object(Map::new()));
    run_tool(s, name, &args)
}

pub fn specs() -> Vec<crate::commands::CommandSpec> {
    vec![crate::commands::CommandSpec {
        id: "ai.tool",
        label: "Run Assistant Tool",
        menu: &[],
        shortcut: None,
        params: r##"{"name": "search_commands|run_command|inspect_document", "args?": object}"##,
        enabled: crate::commands::always,
        run: ai_tool,
        journal: false,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prefs::AiProvider;

    fn req(protocol: Protocol) -> AiRequest {
        AiRequest {
            provider: AiProvider::Preset,
            protocol,
            base_url: "https://x.test/v1".into(),
            model: "m".into(),
            headers: vec![("Authorization".into(), "Bearer k".into())],
        }
    }

    fn conversation() -> Vec<Message> {
        vec![
            Message::User { text: "make it warmer".into(), images: vec!["AAAA".into()] },
            Message::Assistant {
                text: String::new(),
                tool_calls: vec![
                    ToolCall { id: "c1".into(), name: "run_command".into(), args: json!({"id": "adjust.warm"}) },
                    ToolCall { id: "c2".into(), name: CANVAS_TOOL.into(), args: json!({}) },
                ],
            },
            Message::ToolResult { call_id: "c1".into(), name: "run_command".into(), content: "{\"ok\":true}".into(), images: vec![] },
            Message::ToolResult { call_id: "c2".into(), name: CANVAS_TOOL.into(), content: "attached".into(), images: vec!["BBBB".into()] },
            Message::Assistant { text: "done".into(), tool_calls: vec![] },
        ]
    }

    #[test]
    fn every_protocol_serialises_the_same_conversation() {
        let a = build_request(&req(Protocol::Anthropic), "sys", &conversation(), 1024);
        assert_eq!(a.url, "https://x.test/v1/messages");
        assert!(a.headers.iter().any(|(k, v)| k == "Content-Type" && v == "application/json"));
        let body: Value = serde_json::from_str(&a.body).unwrap();
        assert_eq!(body["system"], "sys");
        assert_eq!(body["tools"].as_array().map(Vec::len), Some(4));
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 4, "the two tool results share one user message: {msgs:?}");
        assert_eq!(msgs[0]["content"][0]["type"], "image");
        assert_eq!(msgs[1]["content"][0]["type"], "tool_use");
        assert_eq!(msgs[2]["content"].as_array().map(Vec::len), Some(2));
        assert_eq!(msgs[2]["content"][1]["content"][1]["type"], "image");

        let o = build_request(&req(Protocol::OpenAi), "sys", &conversation(), 1024);
        assert_eq!(o.url, "https://x.test/v1/chat/completions");
        let body: Value = serde_json::from_str(&o.body).unwrap();
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs[1]["content"][1]["image_url"]["url"].as_str().unwrap(), "data:image/png;base64,AAAA");
        assert_eq!(msgs[2]["tool_calls"][0]["function"]["arguments"], "{\"id\":\"adjust.warm\"}");
        assert_eq!(msgs[3]["role"], "tool");
        assert_eq!(msgs[5]["role"], "user", "the canvas picture follows the tool message");
        assert_eq!(body["tools"][0]["type"], "function");

        let g = build_request(&req(Protocol::Google), "sys", &conversation(), 1024);
        assert_eq!(g.url, "https://x.test/v1/models/m:generateContent");
        let body: Value = serde_json::from_str(&g.body).unwrap();
        assert_eq!(body["system_instruction"]["parts"][0]["text"], "sys");
        let c = body["contents"].as_array().unwrap();
        assert_eq!(c[1]["role"], "model");
        assert_eq!(c[1]["parts"][0]["functionCall"]["name"], "run_command");
        assert_eq!(c[2]["parts"].as_array().map(Vec::len), Some(3), "two responses and one picture in one user turn");
        assert_eq!(body["tools"][0]["function_declarations"].as_array().map(Vec::len), Some(4));
    }

    #[test]
    fn replies_and_errors_parse_for_every_protocol() {
        let a = parse_response(Protocol::Anthropic, 200, r#"{"content":[{"type":"text","text":"hi"},{"type":"tool_use","id":"t1","name":"run_command","input":{"id":"file.new"}}],"stop_reason":"tool_use"}"#).unwrap();
        assert_eq!((a.text.as_str(), a.stop.as_str()), ("hi", "tool_use"));
        assert_eq!(a.tool_calls[0].args["id"], "file.new");
        let o = parse_response(Protocol::OpenAi, 200, r#"{"choices":[{"message":{"content":null,"tool_calls":[{"id":"x","type":"function","function":{"name":"inspect_document","arguments":"{}"}}]},"finish_reason":"tool_calls"}]}"#).unwrap();
        assert_eq!(o.tool_calls[0].name, "inspect_document");
        assert_eq!(o.tool_calls[0].args, json!({}));
        let o2 = parse_response(Protocol::OpenAi, 200, r#"{"choices":[{"message":{"content":"plain"},"finish_reason":"stop"}]}"#).unwrap();
        assert_eq!(o2.text, "plain");
        let g = parse_response(Protocol::Google, 200, r#"{"candidates":[{"content":{"parts":[{"functionCall":{"name":"search_commands","args":{"query":"blur"}}},{"text":"looking"}]},"finishReason":"STOP"}]}"#).unwrap();
        assert_eq!(g.tool_calls[0].id, "call-0");
        assert_eq!(g.text, "looking");
        // Errors: a provider error object, an HTTP status without JSON, and broken JSON.
        assert!(
            parse_response(Protocol::Anthropic, 401, r#"{"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key"}}"#)
                .unwrap_err()
                .contains("invalid x-api-key")
        );
        assert!(parse_response(Protocol::OpenAi, 502, "<html>bad gateway</html>").unwrap_err().contains("502"));
        assert!(parse_response(Protocol::Google, 200, "{not json").unwrap_err().contains("not JSON"));
        assert!(parse_response(Protocol::Google, 200, r#"{"promptFeedback":{"blockReason":"SAFETY"}}"#).unwrap_err().contains("SAFETY"));
        // Malformed but well-formed JSON never panics.
        for junk in ["{}", "[]", "null", r#"{"choices":[]}"#, r#"{"content":"x"}"#, r#"{"candidates":[{"content":{"parts":"no"}}]}"#] {
            for p in [Protocol::Anthropic, Protocol::OpenAi, Protocol::Google] {
                let _ = parse_response(p, 200, junk);
            }
        }
    }

    #[test]
    fn tools_search_run_and_inspect_through_the_engine() {
        let mut s = Session::new();
        let hits = run_tool(&mut s, "search_commands", &json!({"query": "new file", "limit": 5})).unwrap();
        let ids: Vec<&str> = hits["commands"].as_array().unwrap().iter().filter_map(|c| c["id"].as_str()).collect();
        assert!(ids.contains(&"file.new"), "{ids:?}");
        assert!(hits["commands"][0]["params"].is_string());
        let r = run_tool(&mut s, "run_command", &json!({"id": "file.new", "params": {"width": 32, "height": 16}})).unwrap();
        assert_eq!(r["ok"], true, "{r}");
        let d = run_tool(&mut s, "inspect_document", &json!({})).unwrap();
        assert_eq!(d["document"]["width"], 32);
        let bad = run_tool(&mut s, "run_command", &json!({"id": "file.nope"})).unwrap();
        assert_eq!(bad["ok"], false);
        assert!(bad["didYouMean"].as_array().is_some());
        let failed = run_tool(&mut s, "run_command", &json!({"id": "image.resize", "params": {"width": -4}})).unwrap();
        assert_eq!(failed["ok"], false, "{failed}");
        assert!(run_tool(&mut s, "run_command", &json!({})).is_err());
        assert!(run_tool(&mut s, CANVAS_TOOL, &json!({})).is_err());
        assert_eq!(run_tool(&mut s, "fly", &json!({})).unwrap()["ok"], false);
        // The same through the command.
        let v = s.execute("ai.tool", json!({"name": "search_commands", "args": {"query": "gaussian"}})).unwrap();
        assert!(v["commands"].as_array().is_some_and(|c| !c.is_empty()));
        assert!(s.execute("ai.tool", json!({})).is_err());
        let prompt = system_prompt(&s);
        assert!(prompt.contains("shepherd") && prompt.contains("32 x 16"));
        assert!(tool_text(&json!({"a": "b"})).contains("\"a\""));
        let long = tool_text(&Value::String("é".repeat(MAX_TOOL_TEXT)));
        assert!(long.contains("[cut:"));
    }
}
