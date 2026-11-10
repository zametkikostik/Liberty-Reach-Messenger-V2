//! Liberty Messenger desktop / CLI entry

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "liberty_messenger=info".into()),
        )
        .init();

    tracing::info!("Liberty Messenger V2 {}", env!("CARGO_PKG_VERSION"));

    use liberty_messenger::crypto::HybridCipher;
    let keys = HybridCipher::generate_session_key();
    let ct = HybridCipher::encrypt(&keys, b"hello liberty").expect("encrypt");
    let pt = HybridCipher::decrypt(&keys, &ct).expect("decrypt");
    assert_eq!(pt, b"hello liberty");
    tracing::info!("Crypto self-check OK");
}
