//! Live AI translate — requires API key and `--features ai`.
//!
//! ```bash
//! export QWEN_API_KEY=sk-...
//! cargo run -p liberty-messenger --example ai_translate --features ai -- "Hello, world" ru
//! ```

use liberty_messenger::ai::{AiAssistant, TranslationRequest};

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let text = args.next().unwrap_or_else(|| "Hello from Liberty".into());
    let target = args.next().unwrap_or_else(|| "ru".into());

    let ai = AiAssistant::new(None);
    if !ai.is_enabled() {
        eprintln!("Set QWEN_API_KEY (or OPENAI_API_KEY) and build with --features ai");
        std::process::exit(1);
    }

    match ai
        .translate(TranslationRequest {
            text,
            source_lang: None,
            target_lang: target,
        })
        .await
    {
        Ok(r) => println!("{}", r.translated),
        Err(e) => {
            eprintln!("AI error: {e}");
            std::process::exit(2);
        }
    }
}
