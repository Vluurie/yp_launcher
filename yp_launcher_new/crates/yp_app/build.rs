fn version_part(name: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|part| !part.is_empty())
        .unwrap_or_else(|| "0".to_owned())
}

fn main() {
    println!("cargo:rerun-if-changed=yp_app.rc");
    println!("cargo:rerun-if-changed=../../assets/icon/app_icon.ico");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let macros = [
            format!("YP_MAJOR={}", version_part("CARGO_PKG_VERSION_MAJOR")),
            format!("YP_MINOR={}", version_part("CARGO_PKG_VERSION_MINOR")),
            format!("YP_PATCH={}", version_part("CARGO_PKG_VERSION_PATCH")),
        ];
        let result = embed_resource::compile("yp_app.rc", &macros);
        if let Err(error) = result.manifest_optional() {
            panic!("could not embed the application icon and version details: {error}");
        }
    }
}
