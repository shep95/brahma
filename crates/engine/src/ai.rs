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

/// Everything a caller needs to talk to the configured provider, built from the preferences.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiRequest {
    pub provider: AiProvider,
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
        AiProvider::Custom | AiProvider::None => "",
    }
}

/// The default API base URL per provider.
pub fn default_base_url(provider: AiProvider) -> &'static str {
    match provider {
        AiProvider::Anthropic => "https://api.anthropic.com/v1",
        AiProvider::OpenAi => "https://api.openai.com/v1",
        AiProvider::Google => "https://generativelanguage.googleapis.com/v1beta",
        AiProvider::Custom | AiProvider::None => "",
    }
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
        let model = trimmed(&integrations.ai_model).unwrap_or(default_model(provider)).to_string();
        let (base_url, headers) = match provider {
            AiProvider::None => return None,
            AiProvider::Anthropic => {
                let key = trimmed(&integrations.anthropic_api_key)?;
                (default_base_url(provider).to_string(), vec![("x-api-key".into(), key.into()), ("anthropic-version".into(), "2023-06-01".into())])
            }
            AiProvider::OpenAi => {
                let key = trimmed(&integrations.openai_api_key)?;
                (default_base_url(provider).to_string(), vec![("Authorization".into(), format!("Bearer {key}"))])
            }
            AiProvider::Google => {
                let key = trimmed(&integrations.google_api_key)?;
                (default_base_url(provider).to_string(), vec![("x-goog-api-key".into(), key.into())])
            }
            AiProvider::Custom => {
                let base = trimmed(&integrations.custom_base_url)?.trim_end_matches('/').to_string();
                let headers = trimmed(&integrations.custom_api_key).map(|k| vec![("Authorization".into(), format!("Bearer {k}"))]).unwrap_or_default();
                (base, headers)
            }
        };
        Some(AiRequest { provider, base_url, model, headers })
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
        "keys": {
            "anthropic": trimmed(&i.anthropic_api_key).is_some(),
            "openai": trimmed(&i.openai_api_key).is_some(),
            "google": trimmed(&i.google_api_key).is_some(),
            "custom": trimmed(&i.custom_api_key).is_some(),
        },
        "keysStayOnDevice": true,
        "signUp": false,
    }))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![CommandSpec { id: "ai.status", label: "AI Status", menu: &[], shortcut: None, params: r##"{}"##, enabled: always, run: ai_status, journal: false }]
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
    fn a_custom_endpoint_needs_only_a_base_url() {
        let mut s = Session::new();
        s.execute("prefs.set", json!({"values": {"integrations.aiProvider": "custom", "integrations.customBaseUrl": "http://localhost:11434/v1/", "integrations.aiModel": "llama"}})).unwrap();
        let r = s.ai_request().unwrap();
        assert_eq!(r.base_url, "http://localhost:11434/v1");
        assert_eq!(r.model, "llama");
        assert!(r.headers.is_empty());
    }
}
