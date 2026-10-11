use std::path::PathBuf;

fn main() {
    let windows = tauri_build::WindowsAttributes::new()
        .window_icon_path(PathBuf::from("../tauri-webview/icons/icon.ico"));
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("failed to generate Tauri context");
}
