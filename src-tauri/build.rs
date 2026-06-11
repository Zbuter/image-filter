fn main() {
    tauri_build::build();

    // Point ort-sys to bundled ONNX Runtime libraries
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let ort_base = format!("{}/onnxruntime", manifest_dir);

    let lib_dir = match target_os.as_str() {
        "windows" => format!("{}/win-x64", ort_base),
        "macos" => format!("{}/osx-universal2", ort_base),
        "linux" => format!("{}/linux-x64", ort_base),
        _ => return,
    };

    println!("cargo:rustc-env=ORT_LIB_LOCATION={}", lib_dir);
    println!("cargo:rustc-env=ORT_STRATEGY=system");
}