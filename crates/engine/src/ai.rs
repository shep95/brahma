//! Bring-your-own-key AI access.
//!
//! brahma stays free and has no account or sign-up of its own: anyone who wants AI features brings
//! their own API key, entered in Edit › Preferences › AI Integrations. The keys are saved only in
//! the preferences file on the user's device (`localStorage` in the browser) and are sent to no
//! one but the provider the user picked. This module turns those preferences into a ready-to-use
//! request shape ([`Session::ai_request`]) and answers `ai.status` for the UI, the CLI and agents,
//! which never reveals the keys themselves.

use serde_json::{Value, json};

use crate::commands::{CommandSpec, always};
use crate::prefs::AiProvider;
use crate::{Result, Session};

/// The wire protocol a provider speaks. Nearly every platform in the world speaks OpenAI's chat
/// completions shape; Anthropic and Google have their own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Protocol {
    Anthropic,
    OpenAi,
    Google,
}

/// A ready-made provider: where it lives, how it is spoken to, and a sensible default model.
/// Keys always come from the user; brahma ships none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    /// ISO 3166 country code of the company, so a user can pick a provider in their own
    /// jurisdiction.
    pub country: &'static str,
    pub protocol: Protocol,
    pub base_url: &'static str,
    pub default_model: &'static str,
    /// Where the key goes: `bearer` (Authorization: Bearer), or a named header.
    pub key_header: &'static str,
}

/// Every major AI platform brahma knows how to talk to, by id. Any other OpenAI-compatible
/// host (a local server, a proxy, a vendor not listed) works through the `custom` provider.
pub const PRESETS: &[Preset] = &[
    Preset {
        id: "anthropic",
        name: "Anthropic (Claude)",
        country: "US",
        protocol: Protocol::Anthropic,
        base_url: "https://api.anthropic.com/v1",
        default_model: "claude-sonnet-5-5",
        key_header: "x-api-key",
    },
    Preset {
        id: "openai",
        name: "OpenAI (GPT)",
        country: "US",
        protocol: Protocol::OpenAi,
        base_url: "https://api.openai.com/v1",
        default_model: "gpt-5",
        key_header: "bearer",
    },
    Preset {
        id: "google",
        name: "Google (Gemini)",
        country: "US",
        protocol: Protocol::Google,
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        default_model: "gemini-2.5-flash",
        key_header: "x-goog-api-key",
    },
    Preset {
        id: "xai",
        name: "xAI (Grok)",
        country: "US",
        protocol: Protocol::OpenAi,
        base_url: "https://api.x.ai/v1",
        default_model: "grok-4",
        key_header: "bearer",
    },
    Preset {
        id: "openrouter",
        name: "OpenRouter (every model)",
        country: "US",
        protocol: Protocol::OpenAi,
        base_url: "https://openrouter.ai/api/v1",
        default_model: "anthropic/claude-sonnet-4.5",
        key_header: "bearer",
    },
    Preset {
        id: "groq",
        name: "Groq",
        country: "US",
        protocol: Protocol::OpenAi,
        base_url: "https://api.groq.com/openai/v1",
        default_model: "llama-3.3-70b-versatile",
        key_header: "bearer",
    },
    Preset {
        id: "together",
        name: "Together AI",
        country: "US",
        protocol: Protocol::OpenAi,
        base_url: "https://api.together.xyz/v1",
        default_model: "meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8",
        key_header: "bearer",
    },
    Preset {
        id: "perplexity",
        name: "Perplexity",
        country: "US",
        protocol: Protocol::OpenAi,
        base_url: "https://api.perplexity.ai",
        default_model: "sonar-pro",
        key_header: "bearer",
    },
    Preset {
        id: "mistral",
        name: "Mistral AI",
        country: "FR",
        protocol: Protocol::OpenAi,
        base_url: "https://api.mistral.ai/v1",
        default_model: "mistral-large-latest",
        key_header: "bearer",
    },
    Preset {
        id: "deepseek",
        name: "DeepSeek",
        country: "CN",
        protocol: Protocol::OpenAi,
        base_url: "https://api.deepseek.com/v1",
        default_model: "deepseek-chat",
        key_header: "bearer",
    },
    Preset {
        id: "qwen",
        name: "Alibaba Qwen (DashScope)",
        country: "CN",
        protocol: Protocol::OpenAi,
        base_url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
        default_model: "qwen-vl-max",
        key_header: "bearer",
    },
    Preset {
        id: "moonshot",
        name: "Moonshot (Kimi)",
        country: "CN",
        protocol: Protocol::OpenAi,
        base_url: "https://api.moonshot.ai/v1",
        default_model: "kimi-k2-0905-preview",
        key_header: "bearer",
    },
    Preset {
        id: "zhipu",
        name: "Zhipu (GLM)",
        country: "CN",
        protocol: Protocol::OpenAi,
        base_url: "https://api.z.ai/api/paas/v4",
        default_model: "glm-4.5v",
        key_header: "bearer",
    },
    Preset {
        id: "minimax",
        name: "MiniMax",
        country: "CN",
        protocol: Protocol::OpenAi,
        base_url: "https://api.minimax.io/v1",
        default_model: "MiniMax-M2",
        key_header: "bearer",
    },
    Preset {
        id: "stepfun",
        name: "StepFun",
        country: "CN",
        protocol: Protocol::OpenAi,
        base_url: "https://api.stepfun.com/v1",
        default_model: "step-3",
        key_header: "bearer",
    },
    Preset {
        id: "yandex",
        name: "Yandex (YandexGPT)",
        country: "RU",
        protocol: Protocol::OpenAi,
        base_url: "https://llm.api.cloud.yandex.net/v1",
        default_model: "yandexgpt/latest",
        key_header: "Api-Key",
    },
    Preset {
        id: "sarvam",
        name: "Sarvam AI",
        country: "IN",
        protocol: Protocol::OpenAi,
        base_url: "https://api.sarvam.ai/v1",
        default_model: "sarvam-m",
        key_header: "bearer",
    },
    Preset {
        id: "upstage",
        name: "Upstage (Solar)",
        country: "KR",
        protocol: Protocol::OpenAi,
        base_url: "https://api.upstage.ai/v1",
        default_model: "solar-pro2",
        key_header: "bearer",
    },
    Preset {
        id: "naver",
        name: "Naver (HyperCLOVA X)",
        country: "KR",
        protocol: Protocol::OpenAi,
        base_url: "https://clovastudio.stream.ntruss.com/v1/openai",
        default_model: "HCX-005",
        key_header: "bearer",
    },
    Preset {
        id: "ai21",
        name: "AI21 Labs (Jamba)",
        country: "IL",
        protocol: Protocol::OpenAi,
        base_url: "https://api.ai21.com/studio/v1",
        default_model: "jamba-large",
        key_header: "bearer",
    },
    Preset {
        id: "cohere",
        name: "Cohere",
        country: "CA",
        protocol: Protocol::OpenAi,
        base_url: "https://api.cohere.ai/compatibility/v1",
        default_model: "command-a-03-2025",
        key_header: "bearer",
    },
    Preset {
        id: "aleph",
        name: "Aleph Alpha",
        country: "DE",
        protocol: Protocol::OpenAi,
        base_url: "https://api.aleph-alpha.com/v1",
        default_model: "llama-3.3-70b-instruct",
        key_header: "bearer",
    },
    Preset {
        id: "maritaca",
        name: "Maritaca (Sabiá)",
        country: "BR",
        protocol: Protocol::OpenAi,
        base_url: "https://chat.maritaca.ai/api",
        default_model: "sabia-3",
        key_header: "bearer",
    },
    Preset {
        id: "nvidia",
        name: "NVIDIA NIM",
        country: "US",
        protocol: Protocol::OpenAi,
        base_url: "https://integrate.api.nvidia.com/v1",
        default_model: "meta/llama-4-maverick-17b-128e-instruct",
        key_header: "bearer",
    },
    Preset {
        id: "fireworks",
        name: "Fireworks AI",
        country: "US",
        protocol: Protocol::OpenAi,
        base_url: "https://api.fireworks.ai/inference/v1",
        default_model: "accounts/fireworks/models/llama4-maverick-instruct-basic",
        key_header: "bearer",
    },
    Preset {
        id: "cerebras",
        name: "Cerebras",
        country: "US",
        protocol: Protocol::OpenAi,
        base_url: "https://api.cerebras.ai/v1",
        default_model: "llama-3.3-70b",
        key_header: "bearer",
    },
    Preset {
        id: "ollama",
        name: "Ollama (local, no key)",
        country: "--",
        protocol: Protocol::OpenAi,
        base_url: "http://localhost:11434/v1",
        default_model: "llama3.2-vision",
        key_header: "bearer",
    },
    Preset {
        id: "lmstudio",
        name: "LM Studio (local, no key)",
        country: "--",
        protocol: Protocol::OpenAi,
        base_url: "http://localhost:1234/v1",
        default_model: "",
        key_header: "bearer",
    },
];

/// The preset with this id.
pub fn preset(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.id == id.trim())
}

/// Everything a caller needs to talk to the configured provider, built from the preferences.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiRequest {
    pub provider: AiProvider,
    /// How to shape the request body (see [`crate::ai_chat`]).
    pub protocol: Protocol,
    /// Base URL of the provider's HTTP API (no trailing slash).
    pub base_url: String,
    /// Model name to send.
    pub model: String,
    /// HTTP headers carrying the user's key and the provider's required version headers.
    pub headers: Vec<(String, String)>,
}

/// The default model per provider when the preference is empty.
pub fn default_model(provider: AiProvider) -> &'static str {
    match provider {
        AiProvider::Anthropic => "claude-sonnet-5-5",
        AiProvider::OpenAi => "gpt-5",
        AiProvider::Google => "gemini-2.5-flash",
        AiProvider::Custom | AiProvider::None | AiProvider::Preset => "",
    }
}

/// The default API base URL per provider.
pub fn default_base_url(provider: AiProvider) -> &'static str {
    match provider {
        AiProvider::Anthropic => "https://api.anthropic.com/v1",
        AiProvider::OpenAi => "https://api.openai.com/v1",
        AiProvider::Google => "https://generativelanguage.googleapis.com/v1beta",
        AiProvider::Custom | AiProvider::None | AiProvider::Preset => "",
    }
}

/// Anthropic's headers: the key, the API version, and the opt-in that lets a browser page (the
/// web build) call the API directly with the user's own key.
fn anthropic_headers(key: &str) -> Vec<(String, String)> {
    vec![
        ("x-api-key".into(), key.into()),
        ("anthropic-version".into(), "2023-06-01".into()),
        ("anthropic-dangerous-direct-browser-access".into(), "true".into()),
    ]
}

fn trimmed(s: &str) -> Option<&str> {
    let t = s.trim();
    (!t.is_empty()).then_some(t)
}

impl Session {
    /// The request shape for the configured provider, or `None` while no provider is picked or
    /// the picked provider has no key (a custom endpoint needs only a base URL).
    pub fn ai_request(&self) -> Option<AiRequest> {
        let integrations = &self.prefs().integrations;
        let provider = integrations.ai_provider;
        let mut model = trimmed(&integrations.ai_model).unwrap_or(default_model(provider)).to_string();
        let (protocol, base_url, headers) = match provider {
            AiProvider::None => return None,
            AiProvider::Anthropic => {
                let key = trimmed(&integrations.anthropic_api_key)?;
                (Protocol::Anthropic, default_base_url(provider).to_string(), anthropic_headers(key))
            }
            AiProvider::OpenAi => {
                let key = trimmed(&integrations.openai_api_key)?;
                (Protocol::OpenAi, default_base_url(provider).to_string(), vec![("Authorization".into(), format!("Bearer {key}"))])
            }
            AiProvider::Google => {
                let key = trimmed(&integrations.google_api_key)?;
                (Protocol::Google, default_base_url(provider).to_string(), vec![("x-goog-api-key".into(), key.into())])
            }
            AiProvider::Custom => {
                let base = trimmed(&integrations.custom_base_url)?.trim_end_matches('/').to_string();
                let headers = trimmed(&integrations.custom_api_key).map(|k| vec![("Authorization".into(), format!("Bearer {k}"))]).unwrap_or_default();
                (Protocol::OpenAi, base, headers)
            }
            AiProvider::Preset => {
                let p = preset(&integrations.ai_preset)?;
                let local = p.base_url.starts_with("http://localhost");
                let key = trimmed(&integrations.preset_api_key);
                if key.is_none() && !local {
                    return None;
                }
                if model.is_empty() {
                    model = p.default_model.to_string();
                }
                let mut headers: Vec<(String, String)> = Vec::new();
                if let Some(k) = key {
                    match p.key_header {
                        "bearer" => headers.push(("Authorization".into(), format!("Bearer {k}"))),
                        h => headers.push((h.into(), k.into())),
                    }
                }
                if p.protocol == Protocol::Anthropic {
                    headers.extend(anthropic_headers("").into_iter().skip(1));
                }
                (p.protocol, p.base_url.to_string(), headers)
            }
        };
        Some(AiRequest { provider, protocol, base_url, model, headers })
    }
}

/// `ai.status`: which provider is configured and whether it is ready, without the keys.
fn ai_status(s: &mut Session, _: &Value) -> Result<Value> {
    let i = &s.prefs().integrations;
    let req = s.ai_request();
    Ok(json!({
        "provider": i.ai_provider.name(),
        "ready": req.is_some(),
        "model": req.as_ref().map(|r| r.model.clone()).unwrap_or_default(),
        "baseUrl": req.as_ref().map(|r| r.base_url.clone()).unwrap_or_default(),
        "preset": i.ai_preset,
        "protocol": req.as_ref().map(|r| r.protocol),
        "keys": {
            "anthropic": trimmed(&i.anthropic_api_key).is_some(),
            "openai": trimmed(&i.openai_api_key).is_some(),
            "google": trimmed(&i.google_api_key).is_some(),
            "custom": trimmed(&i.custom_api_key).is_some(),
            "preset": trimmed(&i.preset_api_key).is_some(),
        },
        "keysStayOnDevice": true,
        "signUp": false,
    }))
}

/// `ai.providers`: every ready-made provider (id, name, country, protocol, base url, default
/// model), for pickers and agents. Keys are never part of it.
fn ai_providers(_: &mut Session, _: &Value) -> Result<Value> {
    Ok(json!({"providers": PRESETS}))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec { id: "ai.status", label: "AI Status", menu: &[], shortcut: None, params: r##"{}"##, enabled: always, run: ai_status, journal: false },
        CommandSpec {
            id: "ai.providers",
            label: "AI Providers",
            menu: &[],
            shortcut: None,
            params: r##"{}"##,
            enabled: always,
            run: ai_providers,
            journal: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn no_provider_means_no_request_and_status_says_so() {
        let mut s = Session::new();
        assert!(s.ai_request().is_none());
        let st = s.execute("ai.status", json!({})).unwrap();
        assert_eq!(st["provider"], "none");
        assert_eq!(st["ready"], false);
        assert_eq!(st["keysStayOnDevice"], true);
        assert_eq!(st["signUp"], false);
    }

    #[test]
    fn a_key_in_the_preferences_builds_the_request_but_never_leaks_through_status() {
        let mut s = Session::new();
        s.execute("prefs.set", json!({"values": {"integrations.aiProvider": "anthropic", "integrations.anthropicApiKey": "sk-ant-secret"}})).unwrap();
        let r = s.ai_request().unwrap();
        assert_eq!(r.provider, AiProvider::Anthropic);
        assert_eq!(r.model, default_model(AiProvider::Anthropic));
        assert!(r.headers.iter().any(|(k, v)| k == "x-api-key" && v == "sk-ant-secret"));
        let st = s.execute("ai.status", json!({})).unwrap();
        assert_eq!(st["ready"], true);
        assert_eq!(st["keys"]["anthropic"], true);
        assert!(!st.to_string().contains("sk-ant-secret"));
    }

    #[test]
    fn presets_cover_every_region_and_build_their_own_headers() {
        let mut ids = std::collections::HashSet::new();
        for p in PRESETS {
            assert!(ids.insert(p.id), "duplicate preset {}", p.id);
            assert!(p.base_url.starts_with("http"), "{}", p.id);
            assert!(!p.base_url.ends_with('/'), "{}", p.id);
        }
        for country in ["US", "FR", "CN", "RU", "IN", "KR", "IL", "CA", "DE", "BR"] {
            assert!(PRESETS.iter().any(|p| p.country == country), "no provider in {country}");
        }
        let mut s = Session::new();
        let v = s.execute("ai.providers", json!({})).unwrap();
        assert_eq!(v["providers"].as_array().map(Vec::len), Some(PRESETS.len()));
        assert!(!v.to_string().contains("ApiKey"));
        // A preset without a key is not ready, except a local server.
        s.execute("prefs.set", json!({"values": {"integrations.aiProvider": "preset", "integrations.aiPreset": "deepseek"}})).unwrap();
        assert!(s.ai_request().is_none());
        s.execute("prefs.set", json!({"values": {"integrations.presetApiKey": "sk-deep"}})).unwrap();
        let r = s.ai_request().unwrap();
        assert_eq!((r.protocol, r.base_url.as_str(), r.model.as_str()), (Protocol::OpenAi, "https://api.deepseek.com/v1", "deepseek-chat"));
        assert_eq!(r.headers, vec![("Authorization".to_string(), "Bearer sk-deep".to_string())]);
        s.execute("prefs.set", json!({"values": {"integrations.aiPreset": "yandex"}})).unwrap();
        assert_eq!(s.ai_request().unwrap().headers, vec![("Api-Key".to_string(), "sk-deep".to_string())]);
        s.execute("prefs.set", json!({"values": {"integrations.aiPreset": "ollama", "integrations.presetApiKey": ""}})).unwrap();
        assert!(s.ai_request().is_some(), "local servers need no key");
        s.execute("prefs.set", json!({"values": {"integrations.aiPreset": "nowhere"}})).unwrap();
        assert!(s.ai_request().is_none());
        let st = s.execute("ai.status", json!({})).unwrap();
        assert_eq!(st["preset"], "nowhere");
    }

    #[test]
    fn a_custom_endpoint_needs_only_a_base_url() {
        let mut s = Session::new();
        s.execute("prefs.set", json!({"values": {"integrations.aiProvider": "custom", "integrations.customBaseUrl": "http://localhost:11434/v1/", "integrations.aiModel": "llama"}})).unwrap();
        let r = s.ai_request().unwrap();
        assert_eq!(r.base_url, "http://localhost:11434/v1");
        assert_eq!(r.model, "llama");
        assert!(r.headers.is_empty());
    }
}
