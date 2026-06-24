fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let config = cbindgen::Config::from_file("cbindgen.toml").expect("cbindgen config");
    let bindings = cbindgen::generate_with_config(&crate_dir, config).expect("generate bindings");

    let out_dir = std::path::Path::new(&crate_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("shim source root")
        .join("include");
    std::fs::create_dir_all(&out_dir).expect("create include dir");
    bindings.write_to_file(out_dir.join("civis_ffi.h"));
}
