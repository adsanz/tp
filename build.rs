fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();

        // Convert PNG to ICO
        let icon_path = "icon.png";
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let ico_path = std::path::Path::new(&out_dir).join("icon.ico");

        if let Ok(img) = image::open(icon_path) {
            // Resize to 256x256 which is the max for ICO
            let img = img.resize(256, 256, image::imageops::FilterType::Lanczos3);

            if let Err(e) = img.save_with_format(&ico_path, image::ImageFormat::Ico) {
                println!("cargo:warning=Failed to save icon.ico: {}", e);
            } else {
                res.set_icon(ico_path.to_str().unwrap());
            }
        } else {
            println!("cargo:warning=Failed to open icon.png");
        }

        // Explicitly set the windres executable path since we are cross-compiling
        // and the auto-detection might fail or look for the wrong binary name.
        // if build from linux to windows
        if std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "gnu" {
            res.set_windres_path("x86_64-w64-mingw32-windres");
        }

        if let Err(e) = res.compile() {
            println!("cargo:warning=Failed to compile Windows resources: {}", e);
        }
    }
}
