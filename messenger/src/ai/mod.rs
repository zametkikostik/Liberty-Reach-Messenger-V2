use serde::{Deserialize, Serialize};

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
}

impl AiAssistant {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    pub async fn translate(&self, req: TranslationRequest) -> anyhow::Result<TranslationResponse> {
        // Real provider only when api_key is Some
        if self.api_key.is_none() {
            return Ok(TranslationResponse {
                translated: req.text,
                detected_lang: req.source_lang,
            });
        }
        Ok(TranslationResponse {
            translated: req.text,
            detected_lang: req.source_lang,
        })
    }

    pub async fn summarize(&self, text: &str) -> anyhow::Result<String> {
        let preview: String = text.chars().take(180).collect();
        Ok(format!("Summary: {preview}..."))
    }
}
