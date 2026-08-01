fn main() {
    println!("cargo:rerun-if-env-changed=CYRENE_UIACCESS");
    #[cfg(target_os = "windows")]
    {
        let manifest = if std::env::var("CYRENE_UIACCESS").ok().as_deref() == Some("1") {
            include_str!("windows-uiaccess-manifest.xml")
        } else {
            include_str!("windows-app-manifest.xml")
        };
        let attributes = tauri_build::Attributes::new().windows_attributes(
            tauri_build::WindowsAttributes::new().app_manifest(manifest),
        );
        tauri_build::try_build(attributes).expect("failed to build Tauri application");
    }
    #[cfg(not(target_os = "windows"))]
    tauri_build::build();
}
