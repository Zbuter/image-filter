use std::fs;
use std::path::PathBuf;

fn main() {
    tauri_build::build();

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

    let target_dir = PathBuf::from(&manifest_dir)
        .parent()
        .unwrap()
        .join("target");
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let bin_dir = target_dir.join(&profile);

    match target_os.as_str() {
        "macos" => {
            let dylib_src = format!("{}/libonnxruntime.dylib", lib_dir);
            let dylib_dst = bin_dir.join("libonnxruntime.1.21.0.dylib");
            let dylib_unversioned = bin_dir.join("libonnxruntime.dylib");
            let _ = fs::copy(&dylib_src, &dylib_dst);
            let _ = fs::copy(&dylib_src, &dylib_unversioned);
            println!("cargo:rustc-link-search=native={}", bin_dir.display());
            println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
            println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
        }
        "windows" => {
            let dll_src = format!("{}/onnxruntime.dll", lib_dir);
            let dll_dst = bin_dir.join("onnxruntime.dll");
            let _ = fs::copy(&dll_src, &dll_dst);
        }
        "linux" => {
            let so_src = format!("{}/libonnxruntime.so", lib_dir);
            let so_dst = bin_dir.join("libonnxruntime.so");
            let _ = fs::copy(&so_src, &so_dst);
            println!("cargo:rustc-link-search=native={}", bin_dir.display());
        }
        _ => {}
    }
}
