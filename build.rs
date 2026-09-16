fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
        if target_env == "msvc" {
            println!("cargo:rustc-link-arg=/EXPORT:NvOptimusEnablement");
            println!("cargo:rustc-link-arg=/EXPORT:AmdPowerXpressRequestHighPerformance");
        } else {
            println!("cargo:rustc-link-arg=-Wl,--export,NvOptimusEnablement");
            println!("cargo:rustc-link-arg=-Wl,--export,AmdPowerXpressRequestHighPerformance");
        }

        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }
}
