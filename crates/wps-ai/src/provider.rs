//! Pluggable LLM Provider Configuration & Deterministic Local AST Intelligence Engine (`wps-ai`)

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AiProviderKind {
    /// Built-in deterministic AST & statistical engine (zero network, instant offline & CI execution)
    #[default]
    BuiltInLocalEngine,
    /// Google Gemini API (`gemini-2.5-flash`, `gemini-2.5-pro`, etc.)
    GoogleGemini {
        api_key: String,
        model: String,
        endpoint_url: String,
    },
    /// OpenAI-compatible REST endpoint (OpenAI, OpenRouter, vLLM, Together)
    OpenAiCompatible {
        api_key: String,
        model: String,
        base_url: String,
    },
    /// Local Ollama server (`http://localhost:11434`)
    LocalOllama {
        model: String,
        host_url: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmProviderConfig {
    pub provider: AiProviderKind,
    pub temperature: f32,
    pub max_output_tokens: u32,
    pub system_preamble: String,
}

impl Default for LlmProviderConfig {
    fn default() -> Self {
        Self {
            provider: AiProviderKind::BuiltInLocalEngine,
            temperature: 0.2,
            max_output_tokens: 4096,
            system_preamble: "You are WPS AI, an AST-native office productivity co-pilot.".to_string(),
        }
    }
}

/// Represents an outbound HTTP request payload that the Tauri native host or browser `fetch`
/// adapter can execute when connected to an external cloud/local LLM API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboundLlmHttpRequest {
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body_json: String,
}

impl LlmProviderConfig {
    /// Builds a provider-specific JSON HTTP request for external LLM execution when not using
    /// `BuiltInLocalEngine`.
    pub fn build_http_request(&self, user_prompt: &str, json_schema_hint: Option<&str>) -> Option<OutboundLlmHttpRequest> {
        let full_prompt = match json_schema_hint {
            Some(schema) => format!(
                "{}\n\nRespond strictly in valid JSON matching this schema:\n{}\n\nUser Request:\n{}",
                self.system_preamble, schema, user_prompt
            ),
            None => format!("{}\n\nUser Request:\n{}", self.system_preamble, user_prompt),
        };

        match &self.provider {
            AiProviderKind::BuiltInLocalEngine => None,
            AiProviderKind::GoogleGemini {
                api_key,
                model,
                endpoint_url,
            } => {
                let url = format!(
                    "{}/v1beta/models/{}:generateContent?key={}",
                    endpoint_url.trim_end_matches('/'),
                    model,
                    api_key
                );
                let mut headers = BTreeMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                let body = serde_json::json!({
                    "contents": [{ "parts": [{ "text": full_prompt }] }],
                    "generationConfig": {
                        "temperature": self.temperature,
                        "maxOutputTokens": self.max_output_tokens
                    }
                });
                Some(OutboundLlmHttpRequest {
                    url,
                    headers,
                    body_json: body.to_string(),
                })
            }
            AiProviderKind::OpenAiCompatible {
                api_key,
                model,
                base_url,
            } => {
                let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
                let mut headers = BTreeMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
                let body = serde_json::json!({
                    "model": model,
                    "temperature": self.temperature,
                    "max_tokens": self.max_output_tokens,
                    "messages": [
                        { "role": "system", "content": self.system_preamble },
                        { "role": "user", "content": full_prompt }
                    ]
                });
                Some(OutboundLlmHttpRequest {
                    url,
                    headers,
                    body_json: body.to_string(),
                })
            }
            AiProviderKind::LocalOllama { model, host_url } => {
                let url = format!("{}/api/chat", host_url.trim_end_matches('/'));
                let mut headers = BTreeMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                let body = serde_json::json!({
                    "model": model,
                    "stream": false,
                    "messages": [
                        { "role": "system", "content": self.system_preamble },
                        { "role": "user", "content": full_prompt }
                    ]
                });
                Some(OutboundLlmHttpRequest {
                    url,
                    headers,
                    body_json: body.to_string(),
                })
            }
        }
    }
}
