fn main() {
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = std::path::Path::new(&root);

    println!(
        "cargo:rustc-link-arg-bins=--script={}",
        root.join("link.ld").display()
    );
}
