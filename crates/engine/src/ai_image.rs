//! The assistant's `generate_image` tool: a picture from the provider's image model, placed
//! into the open document as a new layer (or opened as a new document when none is open).
//!
//! The request goes to the same provider the chat uses, with its image model: OpenAI-compatible
//! endpoints through `POST /images/generations`, Google through `generateContent` with the
//! image response modality. Anthropic's API has no image model, so the tool says so and the
//! assistant can suggest switching provider for that one step. The shell does the HTTP call
//! (`Services::http`) and hands the body back to [`parse_image_response`] and [`place_generated`].

use crate::ai::{AiRequest, Protocol};
use crate::ai_chat::HttpRequest;
use crate::{EngineError, Result, Session};
use base64::Engine as _;
use serde_json::{Value, json};

/// The tool's name as the model calls it.
pub const IMAGE_TOOL: &str = "generate_image";
/// Shortest and longest side a generated picture may be asked for.
const MIN_SIDE: u32 = 256;
const MAX_SIDE: u32 = 4096;
/// Most bytes decoded from a provider's base64 picture (a 4096² PNG is far below this).
const MAX_IMAGE_BYTES: usize = 64 * 1024 * 1024;
/// Longest prompt sent on.
const MAX_PROMPT_CHARS: usize = 4000;

/// The tool definition in the shape `ai_chat::tools` lists them.
pub fn tool() -> (&'static str, &'static str, Value) {
    (
        IMAGE_TOOL,
        "make a picture from a text prompt with the provider's image model and place it as a new layer of the active document (or open it as a new document when none is open). say what to depict, the style, the mood and the light; ask for `transparent` for a cut-out subject. the result names the layer; call look_at_canvas to judge it.",
        json!({"type": "object", "properties": {
            "prompt": {"type": "string", "description": "what to generate, in plain words"},
            "size": {"type": "string", "description": "width x height in pixels, e.g. '1024x1024' (default), '1536x1024' or '1024x1536'"},
            "transparent": {"type": "boolean", "description": "a transparent background, for a subject to composite (default false)"},
            "name": {"type": "string", "description": "the new layer's name (default: the prompt's first words)"}
        }, "required": ["prompt"]}),
    )
}

/// What the model asked for, validated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageAsk {
    pub prompt: String,
    pub width: u32,
    pub height: u32,
    pub transparent: bool,
    pub name: String,
}

impl ImageAsk {
    pub fn size(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}

/// `args` as an [`ImageAsk`], or why not (the text goes back to the model).
pub fn parse_args(args: &Value) -> std::result::Result<ImageAsk, String> {
    let prompt = args.get("prompt").and_then(Value::as_str).map(str::trim).filter(|p| !p.is_empty()).ok_or("generate_image needs a `prompt`")?;
    let prompt: String = prompt.chars().take(MAX_PROMPT_CHARS).collect();
    let (width, height) = match args.get("size").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()) {
        None => (1024, 1024),
        Some(s) => parse_size(s).ok_or_else(|| format!("`size` must be like '1024x1024' with sides {MIN_SIDE}–{MAX_SIDE}, not {s:?}"))?,
    };
    let transparent = args.get("transparent").and_then(Value::as_bool).unwrap_or(false);
    let name = match args.get("name").and_then(Value::as_str).map(str::trim).filter(|n| !n.is_empty()) {
        Some(n) => n.chars().take(80).collect(),
        None => default_name(&prompt),
    };
    Ok(ImageAsk { prompt, width, height, transparent, name })
}

fn parse_size(s: &str) -> Option<(u32, u32)> {
    let (w, h) = s.to_ascii_lowercase().split_once(['x', '×'])?.pipe(|(w, h)| (w.trim().parse::<u32>().ok(), h.trim().parse::<u32>().ok()));
    let (w, h) = (w?, h?);
    ((MIN_SIDE..=MAX_SIDE).contains(&w) && (MIN_SIDE..=MAX_SIDE).contains(&h)).then_some((w, h))
}

trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}
impl<T> Pipe for T {}

/// The layer name from a prompt: its first six words, title-less and trimmed.
fn default_name(prompt: &str) -> String {
    let words: Vec<&str> = prompt.split_whitespace().take(6).collect();
    let name = words.join(" ");
    let name: String = name.chars().take(48).collect();
    if name.is_empty() { "Generated".into() } else { name }
}

/// The image model for `req`: the preference when set, else the protocol's usual one.
pub fn image_model(req: &AiRequest, preferred: &str) -> std::result::Result<String, String> {
    let p = preferred.trim();
    if !p.is_empty() {
        return Ok(p.to_string());
    }
    match req.protocol {
        Protocol::OpenAi => Ok("gpt-image-1".into()),
        Protocol::Google => Ok("gemini-2.5-flash-image".into()),
        Protocol::Anthropic => Err("this provider has no image model: pick an OpenAI-compatible or Google provider for pictures, or set the image model in the setup rows".into()),
    }
}

/// The provider's image request for `ask`.
pub fn build_image_request(req: &AiRequest, model: &str, ask: &ImageAsk) -> std::result::Result<HttpRequest, String> {
    let mut headers = req.headers.clone();
    headers.push(("Content-Type".into(), "application/json".into()));
    match req.protocol {
        Protocol::OpenAi => {
            let mut body = json!({"model": model, "prompt": ask.prompt, "n": 1, "size": ask.size()});
            if model.starts_with("dall-e") {
                // The older models return a URL unless asked for the bytes.
                body["response_format"] = json!("b64_json");
            } else if ask.transparent {
                body["background"] = json!("transparent");
                body["output_format"] = json!("png");
            }
            Ok(HttpRequest { url: format!("{}/images/generations", req.base_url), headers, body: body.to_string() })
        }
        Protocol::Google => {
            let mut prompt = ask.prompt.clone();
            prompt.push_str(&format!(" (an image of {} by {} pixels", ask.width, ask.height));
            if ask.transparent {
                prompt.push_str(", the subject alone on a plain white background");
            }
            prompt.push(')');
            let body = json!({
                "contents": [{"role": "user", "parts": [{"text": prompt}]}],
                "generationConfig": {"responseModalities": ["IMAGE", "TEXT"]}
            });
            Ok(HttpRequest { url: format!("{}/models/{model}:generateContent", req.base_url), headers, body: body.to_string() })
        }
        Protocol::Anthropic => Err(image_model(req, "").err().unwrap_or_else(|| "this provider has no image model".into())),
    }
}

/// The picture's bytes (PNG, JPEG or WebP as the provider encodes it) from the response, or
/// the provider's error message.
pub fn parse_image_response(protocol: Protocol, status: u16, body: &str) -> std::result::Result<Vec<u8>, String> {
    let v: Value = serde_json::from_str(body).map_err(|_| format!("HTTP {status}: {}", crate::ai_chat::snippet(body)))?;
    if let Some(msg) = v.get("error").and_then(|e| e.get("message").and_then(Value::as_str).or_else(|| e.as_str())) {
        return Err(format!("HTTP {status}: {msg}"));
    }
    if !(200..300).contains(&status) {
        return Err(format!("HTTP {status}: {}", crate::ai_chat::snippet(body)));
    }
    let b64 = match protocol {
        Protocol::OpenAi | Protocol::Anthropic => {
            let first = v.get("data").and_then(Value::as_array).and_then(|d| d.first()).ok_or("the provider returned no image")?;
            if first.get("url").is_some() && first.get("b64_json").is_none() {
                return Err("the provider returned a link instead of the picture's bytes: use a model that answers with b64_json".into());
            }
            first.get("b64_json").and_then(Value::as_str).ok_or("the provider returned no image data")?.to_string()
        }
        Protocol::Google => {
            let parts = v
                .get("candidates")
                .and_then(Value::as_array)
                .and_then(|c| c.first())
                .and_then(|c| c.get("content"))
                .and_then(|c| c.get("parts"))
                .and_then(Value::as_array)
                .ok_or("the provider returned no image")?;
            let inline = parts.iter().find_map(|p| p.get("inlineData").or_else(|| p.get("inline_data")));
            match inline.and_then(|i| i.get("data")).and_then(Value::as_str) {
                Some(d) => d.to_string(),
                None => {
                    let text = parts.iter().filter_map(|p| p.get("text").and_then(Value::as_str)).collect::<Vec<_>>().join(" ");
                    return Err(if text.is_empty() { "the provider returned no image".into() } else { format!("the provider answered in words instead of a picture: {}", crate::ai_chat::snippet(&text)) });
                }
            }
        }
    };
    if b64.len() / 4 * 3 > MAX_IMAGE_BYTES {
        return Err("the generated picture is too large".into());
    }
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64.trim()).map_err(|e| format!("the picture's data could not be decoded: {e}"))?;
    if bytes.is_empty() {
        return Err("the provider returned an empty picture".into());
    }
    Ok(bytes)
}

/// Put the generated picture into the session: a new layer named `name` in the active
/// document (fitted like File › Place), or a new document when none is open. Returns what
/// the model is told: `{placed: "layer"|"document", name, width, height}`.
pub fn place_generated(s: &mut Session, name: &str, bytes: Vec<u8>) -> Result<Value> {
    let file = "generated.png";
    if s.active().is_some() {
        let placed = crate::file_cmds::place_bytes(s, file, bytes, None, &json!({}))?;
        let layer = placed.get("layer").and_then(Value::as_u64).map(brahma_doc::LayerId);
        let bounds = placed.get("bounds").cloned().unwrap_or(Value::Null);
        if let (Some(layer), Some(st)) = (layer, s.active_mut()) {
            let doc = std::sync::Arc::make_mut(&mut st.doc);
            if let Some(l) = doc.layers.iter_mut().find(|l| l.id == layer) {
                l.name = name.to_string();
            }
        }
        return Ok(json!({"placed": "layer", "name": name, "layer": placed.get("layer").cloned().unwrap_or(Value::Null), "bounds": bounds}));
    }
    let imported = brahma_io::import(file, &bytes).map_err(|e| EngineError::Other(format!("the generated picture could not be read: {e}")))?;
    let mut doc = imported.document;
    doc.name = name.to_string();
    let (w, h) = (doc.size.width, doc.size.height);
    s.add_document(doc, None);
    Ok(json!({"placed": "document", "name": name, "width": w, "height": h}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prefs::AiProvider;

    fn req(protocol: Protocol) -> AiRequest {
        AiRequest { provider: AiProvider::Preset, protocol, base_url: "https://x.test/v1".into(), model: "m".into(), headers: vec![("Authorization".into(), "Bearer k".into())] }
    }

    #[test]
    fn arguments_are_validated_and_sizes_bounded() {
        let a = parse_args(&json!({"prompt": "a fox in tall grass at dusk, soft light", "size": "1536x1024", "transparent": true})).unwrap();
        assert_eq!((a.width, a.height, a.transparent), (1536, 1024, true));
        assert_eq!(a.name, "a fox in tall grass at");
        assert_eq!(parse_args(&json!({"prompt": "x"})).unwrap().size(), "1024x1024");
        assert!(parse_args(&json!({})).is_err());
        assert!(parse_args(&json!({"prompt": "   "})).is_err());
        assert!(parse_args(&json!({"prompt": "x", "size": "10x10"})).is_err());
        assert!(parse_args(&json!({"prompt": "x", "size": "99999x1024"})).is_err());
        assert!(parse_args(&json!({"prompt": "x", "size": "wide"})).is_err());
        assert_eq!(parse_args(&json!({"prompt": "x", "size": "1024 × 1024"})).unwrap().size(), "1024x1024");
    }

    #[test]
    fn requests_follow_the_protocol() {
        let ask = parse_args(&json!({"prompt": "a lighthouse", "transparent": true})).unwrap();
        let o = build_image_request(&req(Protocol::OpenAi), "gpt-image-1", &ask).unwrap();
        assert_eq!(o.url, "https://x.test/v1/images/generations");
        let body: Value = serde_json::from_str(&o.body).unwrap();
        assert_eq!(body["background"], "transparent");
        assert_eq!(body["size"], "1024x1024");
        assert!(body.get("response_format").is_none(), "gpt-image rejects response_format");
        let d = build_image_request(&req(Protocol::OpenAi), "dall-e-3", &ask).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&d.body).unwrap()["response_format"], "b64_json");
        let g = build_image_request(&req(Protocol::Google), "gemini-2.5-flash-image", &ask).unwrap();
        assert_eq!(g.url, "https://x.test/v1/models/gemini-2.5-flash-image:generateContent");
        let body: Value = serde_json::from_str(&g.body).unwrap();
        assert_eq!(body["generationConfig"]["responseModalities"][0], "IMAGE");
        assert!(body["contents"][0]["parts"][0]["text"].as_str().unwrap().contains("white background"));
        assert!(build_image_request(&req(Protocol::Anthropic), "x", &ask).is_err());
        assert!(image_model(&req(Protocol::Anthropic), "").is_err());
        assert_eq!(image_model(&req(Protocol::Anthropic), " my-images ").unwrap(), "my-images");
        assert_eq!(image_model(&req(Protocol::OpenAi), "").unwrap(), "gpt-image-1");
    }

    #[test]
    fn responses_yield_bytes_or_the_providers_words() {
        let png = base64::engine::general_purpose::STANDARD.encode(b"\x89PNGfake");
        let o = parse_image_response(Protocol::OpenAi, 200, &json!({"data": [{"b64_json": png}]}).to_string()).unwrap();
        assert_eq!(o, b"\x89PNGfake");
        let g = parse_image_response(Protocol::Google, 200, &json!({"candidates": [{"content": {"parts": [{"text": "here"}, {"inlineData": {"mimeType": "image/png", "data": png}}]}}]}).to_string()).unwrap();
        assert_eq!(g, b"\x89PNGfake");
        let words = parse_image_response(Protocol::Google, 200, &json!({"candidates": [{"content": {"parts": [{"text": "I cannot draw that"}]}}]}).to_string()).unwrap_err();
        assert!(words.contains("I cannot draw that"), "{words}");
        let e = parse_image_response(Protocol::OpenAi, 400, r#"{"error":{"message":"bad prompt"}}"#).unwrap_err();
        assert_eq!(e, "HTTP 400: bad prompt");
        assert!(parse_image_response(Protocol::OpenAi, 200, r#"{"data":[{"url":"https://cdn"}]}"#).unwrap_err().contains("link"));
        assert!(parse_image_response(Protocol::OpenAi, 500, "<html>").unwrap_err().starts_with("HTTP 500"));
        assert!(parse_image_response(Protocol::OpenAi, 200, r#"{"data":[{"b64_json":"!!!"}]}"#).is_err());
        assert!(parse_image_response(Protocol::OpenAi, 200, r#"{"data":[]}"#).is_err());
    }

    #[test]
    fn a_generated_picture_becomes_a_layer_or_a_document() {
        let img = brahma_codecs::Image::from_u8(8, 6, brahma_codecs::ChannelLayout::Rgba, vec![200; 8 * 6 * 4]).unwrap();
        let png = brahma_codecs::encode(&img, brahma_codecs::Format::Png, &brahma_codecs::EncodeOptions::default()).unwrap();
        let mut s = Session::new();
        let v = place_generated(&mut s, "a fox", png.clone()).unwrap();
        assert_eq!(v["placed"], "document");
        assert_eq!(s.active().unwrap().doc.name, "a fox");
        assert_eq!((v["width"].as_u64(), v["height"].as_u64()), (Some(8), Some(6)));
        let before = s.active().unwrap().doc.layers.len();
        let v = place_generated(&mut s, "a second fox", png).unwrap();
        assert_eq!(v["placed"], "layer");
        assert!(v["bounds"].is_array());
        let d = &s.active().unwrap().doc;
        assert_eq!(d.layers.len(), before + 1);
        assert_eq!(d.layers.last().unwrap().name, "a second fox");
        assert!(place_generated(&mut s, "junk", b"not an image".to_vec()).is_err(), "bad bytes fail without a panic");
    }
}
