fn main() {
    let csource = std::env::var("CARGO_FEATURE_CSOURCE").is_ok();
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH")
        .map(|a| a == "wasm32")
        .unwrap_or(false);

    if csource && !wasm {
        // Resolve the repo root relative to this crate's manifest directory.
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        // crates/hyle → repo root is ../../
        let repo_root = std::path::Path::new(&manifest_dir)
            .join("..") .join("..")
            .canonicalize()
            .expect("failed to resolve repo root");

        let hyle_lib  = repo_root.join("lib");
        let qmap_lib  = repo_root.join("../libqmap/lib");

        println!("cargo:rustc-link-search=native={}", hyle_lib.display());
        println!("cargo:rustc-link-search=native={}", qmap_lib.display());
        // Link C libhyle statically to avoid naming collision with the Rust cdylib
        // (which is also called libhyle.so and is placed in target/debug/deps/).
        println!("cargo:rustc-link-lib=static=hyle");
        // qmap has no naming conflict; link dynamically and bake rpath.
        println!("cargo:rustc-link-lib=dylib=qmap");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", qmap_lib.display());
    }
}
