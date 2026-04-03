fn main() {
    println!("cargo:rerun-if-changed=assets/branding/blinkspark.ico");
    println!("cargo:rerun-if-changed=assets/branding/generated/blinkspark.ico");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    #[cfg(windows)]
    {
        use std::path::Path;

        let icon_path = if Path::new("assets/branding/blinkspark.ico").exists() {
            "assets/branding/blinkspark.ico"
        } else if Path::new("assets/branding/generated/blinkspark.ico").exists() {
            "assets/branding/generated/blinkspark.ico"
        } else {
            panic!(
                "Windows icon file not found. Expected assets/branding/blinkspark.ico. \
Run powershell -ExecutionPolicy Bypass -File .\\scripts\\generate_logo_assets.ps1"
            );
        };

        let mut resource = winres::WindowsResource::new();
        resource.set_icon(icon_path);
        resource
            .compile()
            .expect("Failed to compile Windows resources with app icon");
    }
}
