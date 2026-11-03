fn main() {
    // PresentMon is downloaded by CI into bin/ and embedded when present.
    println!("cargo::rustc-check-cfg=cfg(has_presentmon)");
    println!("cargo::rerun-if-changed=bin/PresentMon.exe");
    if std::path::Path::new("bin/PresentMon.exe").exists() {
        println!("cargo::rustc-cfg=has_presentmon");
    }
    tauri_build::build()
}
