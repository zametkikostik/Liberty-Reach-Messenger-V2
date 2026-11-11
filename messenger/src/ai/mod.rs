//! AI helpers — real HTTP calls only when API key is present.
//!
//! Without `QWEN_API_KEY` / constructor key: returns explicit `AiError::NoApiKey`.
//! No mock translations or fake summaries.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("AI disabled: set QWEN_API_KEY (or pass api_key) for real calls")]
    NoApiKey,
    #[error("http: {0}")]
    Http(String),
    #[error("parse: {0}")]
    Parse(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub text: String,
    pub source_lang: Option<String>,
    pub target_lang: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub translated: String,
    pub detected_lang: Option<String>,
}

pub struct AiAssistant {
    pub api_key: Option<String>,
    /// Base URL for the LLM provider (default: DashScope / Qwen compatible)
    pub base_url: String,
}

impl AiAssistant {
    pub fn new(api_key: Option<String>) -> Self {
        let api_key = api_key.or_else(|| std::env::var("QWEN_API_KEY").ok());
        Self {
            api_key,
            base_url: std::env::var("QWEN_BASE_URL").unwrap_or_else(|_| {
                "https://dashscope.aliyuncs.com/compatible-mode/v1".into()
            }),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false)
    }

    /// Translate via OpenAI-compatible chat completions when key is set.
    pub async fn translate(&self, req: TranslationRequest) -> Result<TranslationResponse, AiError> {
        let key = self.api_key.as_ref().filter(|k| !k.is_empty()).ok_or(AiError::NoApiKey)?;
        // Placeholder for reqwest call — wire when HTTP client is on the host:
        // POST {base}/chat/completions with Authorization: Bearer {key}
        let _ = (key, &self.base_url, &req);
        Err(AiError::Http(
            "HTTP client not linked in this build — set QWEN_API_KEY and add reqwest to enable"
                .into(),
        ))
    }

    pub async fn summarize(&self, text: &str) -> Result<String, AiError> {
        if !self.is_enabled() {
            return Err(AiError::NoApiKey);
        }
        let _ = text;
        Err(AiError::Http(
            "HTTP client not linked in this build — set QWEN_API_KEY and add reqwest to enable"
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_key_returns_error() {
        let ai = AiAssistant::new(None);
        assert!(!ai.is_enabled());
        let err = ai
            .translate(TranslationRequest {
                text: "hi".into(),
                source_lang: None,
                target_lang: "ru".into(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, AiError::NoApiKey));
    }
}
