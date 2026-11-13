//! Optional AI proxy — real HTTP only when QWEN_API_KEY / OPENAI_API_KEY is set.
//! Never invents translations. Without a key → 503 with clear body.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use super::AppState;

#[derive(Deserialize)]
pub struct TranslateBody {
    pub text: String,
    #[serde(default)]
    pub source_lang: Option<String>,
    #[serde(default = "default_target")]
    pub target_lang: String,
}

fn default_target() -> String {
    "ru".into()
}

#[derive(Serialize)]
pub struct TranslateResponse {
    pub translated: String,
    pub model: String,
}

#[derive(Serialize)]
struct ChatCompletionsBody<'a> {
    model: &'a str,
    messages: Vec<Msg<'a>>,
    temperature: f32,
}

#[derive(Serialize)]
struct Msg<'a> {
    role: &'a str,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionsReply {
    choices: Option<Vec<Choice>>,
}

#[derive(Deserialize)]
struct Choice {
    message: Option<ChoiceMsg>,
}

#[derive(Deserialize)]
struct ChoiceMsg {
    content: Option<String>,
}

/// POST /api/v1/ai/translate
pub async fn translate(
    State(_state): State<AppState>,
    Json(body): Json<TranslateBody>,
) -> Result<Json<TranslateResponse>, (StatusCode, String)> {
    let key = std::env::var("QWEN_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .ok()
        .filter(|k| !k.is_empty());
    let Some(key) = key else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "AI disabled: set QWEN_API_KEY or OPENAI_API_KEY".into(),
        ));
    };

    if body.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "text required".into()));
    }

    let base = std::env::var("QWEN_BASE_URL").unwrap_or_else(|_| {
        "https://dashscope.aliyuncs.com/compatible-mode/v1".into()
    });
    let model = std::env::var("QWEN_MODEL").unwrap_or_else(|_| "qwen-turbo".into());
    let src = body.source_lang.as_deref().unwrap_or("auto");
    let system = format!(
        "You are a translator. Translate from {src} to {}. Reply with ONLY the translation.",
        body.target_lang
    );

    let url = format!("{}/chat/completions", base.trim_end_matches('/'));
    let payload = ChatCompletionsBody {
        model: &model,
        messages: vec![
            Msg {
                role: "system",
                content: system,
            },
            Msg {
                role: "user",
                content: body.text,
            },
        ],
        temperature: 0.2,
    };

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .bearer_auth(&key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("http: {e}")))?;

    let status = res.status();
    let text = res
        .text()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("read: {e}")))?;
    if !status.is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("provider {status}: {text}"),
        ));
    }

    let parsed: ChatCompletionsReply = serde_json::from_str(&text)
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("parse: {e}")))?;
    let translated = parsed
        .choices
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.message)
        .and_then(|m| m.content)
        .ok_or_else(|| (StatusCode::BAD_GATEWAY, "no choices".into()))?
        .trim()
        .to_string();

    Ok(Json(TranslateResponse {
        translated,
        model,
    }))
}
