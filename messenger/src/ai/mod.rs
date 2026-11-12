//! AI helpers — real HTTP only when an API key is present.
//!
//! * No key → [`AiError::NoApiKey`] (never mock success).
//! * Key set, build **without** `--features ai` → [`AiError::Http`] explaining how to enable.
//! * Key set + `--features ai` → real OpenAI-compatible `chat/completions` (Qwen / DashScope / OpenAI).
//!
//! Env: `QWEN_API_KEY`, `OPENAI_API_KEY`, `QWEN_BASE_URL`, `QWEN_MODEL`

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
struct ChatCompletionsBody<'a> {
    model: &'a str,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatCompletionsReply {
    choices: Option<Vec<ChatChoice>>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: Option<ChatMessage>,
}

pub struct AiAssistant {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
}

impl AiAssistant {
    pub fn new(api_key: Option<String>) -> Self {
        let api_key = api_key
            .or_else(|| std::env::var("QWEN_API_KEY").ok())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok());
        let base_url = std::env::var("QWEN_BASE_URL").unwrap_or_else(|_| {
            "https://dashscope.aliyuncs.com/compatible-mode/v1".into()
        });
        let model = std::env::var("QWEN_MODEL").unwrap_or_else(|_| "qwen-turbo".into());
        Self { api_key, base_url, model }
    }

    pub fn is_enabled(&self) -> bool {
        self.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false)
    }

    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, AiError> {
        let key = self
            .api_key
            .as_ref()
            .filter(|k| !k.is_empty())
            .ok_or(AiError::NoApiKey)?;

        #[cfg(feature = "ai")]
        {
            return self.chat_http(key, messages).await;
        }

        #[cfg(not(feature = "ai"))]
        {
            let _ = (key, messages);
            Err(AiError::Http(
                "build with --features ai (reqwest) and set QWEN_API_KEY for live calls".into(),
            ))
        }
    }

    #[cfg(feature = "ai")]
    async fn chat_http(&self, key: &str, messages: Vec<ChatMessage>) -> Result<String, AiError> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = ChatCompletionsBody { model: &self.model, messages, temperature: 0.2 };
        let client = reqwest::Client::new();
        let res = client
            .post(&url)
            .bearer_auth(key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Http(e.to_string()))?;
        let status = res.status();
        let text = res.text().await.map_err(|e| AiError::Http(e.to_string()))?;
        if !status.is_success() {
            return Err(AiError::Http(format!("{status}: {text}")));
        }
        let parsed: ChatCompletionsReply =
            serde_json::from_str(&text).map_err(|e| AiError::Parse(e.to_string()))?;
        parsed
            .choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message)
            .map(|m| m.content)
            .ok_or_else(|| AiError::Parse("no choices in response".into()))
    }

    pub async fn translate(&self, req: TranslationRequest) -> Result<TranslationResponse, AiError> {
        let src = req.source_lang.as_deref().unwrap_or("auto");
        let system = format!(
            "You are a translator. Translate the user text from {src} to {}. Reply with ONLY the translation, no quotes or commentary.",
            req.target_lang
        );
        let messages = vec![
            ChatMessage { role: "system".into(), content: system },
            ChatMessage { role: "user".into(), content: req.text },
        ];
        let translated = self.chat(messages).await?;
        Ok(TranslationResponse { translated: translated.trim().to_string(), detected_lang: None })
    }

    pub async fn summarize(&self, text: &str) -> Result<String, AiError> {
        if !self.is_enabled() {
            return Err(AiError::NoApiKey);
        }
        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: "Summarize the user message in 1-3 short sentences. No preamble.".into(),
            },
            ChatMessage { role: "user".into(), content: text.to_string() },
        ];
        self.chat(messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_key_errors() {
        let a = AiAssistant {
            api_key: None,
            base_url: "http://localhost".into(),
            model: "x".into(),
        };
        assert!(!a.is_enabled());
        let err = a
            .translate(TranslationRequest {
                text: "hi".into(),
                source_lang: None,
                target_lang: "ru".into(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, AiError::NoApiKey));
    }

    #[tokio::test]
    async fn explicit_empty_key() {
        let a = AiAssistant {
            api_key: Some(String::new()),
            base_url: "http://localhost".into(),
            model: "x".into(),
        };
        assert!(!a.is_enabled());
        let err = a.summarize("x").await.unwrap_err();
        assert!(matches!(err, AiError::NoApiKey));
    }
}
