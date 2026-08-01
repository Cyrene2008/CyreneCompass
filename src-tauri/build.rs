fn main() {
    #[cfg(target_os = "windows")]
    {
        let attributes = tauri_build::Attributes::new().windows_attributes(
            tauri_build::WindowsAttributes::new().app_manifest(include_str!("windows-app-manifest.xml")),
        );
        tauri_build::try_build(attributes).expect("failed to build Tauri application");
    }
    #[cfg(not(target_os = "windows"))]
    tauri_build::build();
}
