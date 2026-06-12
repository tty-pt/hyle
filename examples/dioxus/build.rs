fn main() {
    // When hyle is built with the csource feature (non-wasm), the final binary
    // needs to find libqmap.so at runtime.  Cargo propagates rustc-link-search
    // and rustc-link-lib from dependency build scripts, but not rpath.
    // We re-emit the rpath here so test and binary executables find libqmap.so
    // without LD_LIBRARY_PATH.  libhyle is linked statically so no rpath needed.
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH")
        .map(|a| a == "wasm32")
        .unwrap_or(false);

    if !wasm {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let repo_root = std::path::Path::new(&manifest_dir)
            .join("..").join("..")
            .canonicalize()
            .expect("failed to resolve repo root");

        let qmap_lib = repo_root.join("../libqmap/lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", qmap_lib.display());
    }
}
