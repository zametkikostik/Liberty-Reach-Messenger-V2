# FFI Flutter to Rust

Contract in rust-core/src/api/mod.rs: vault_*, session_*, identity_*, group_*, telegram_import_json, whatsapp_import_text, federation_*, cipher_suites, build_features.

Dart: mobile/lib/services/rust_bridge.dart — tries native lib, else simulation.

Build: ./scripts/build_core.sh and ./scripts/build_android_native.sh
