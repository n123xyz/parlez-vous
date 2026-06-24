const COMMANDS: &[&str] = &["generate_supertonic_tts", "is_supertonic_ready", "download_supertonic_models", "purge_supertonic_models"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
