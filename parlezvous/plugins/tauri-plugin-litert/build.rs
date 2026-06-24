const COMMANDS: &[&str] = &["init_model", "generate_chat", "close_model", "check_model_exists", "download_model", "purge_model", "pick_gallery_image"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
