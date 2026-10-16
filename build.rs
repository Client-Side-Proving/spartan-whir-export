fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    let manifest = std::fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    let dependency = manifest
        .lines()
        .find(|line| line.starts_with("whir-p3 = "))
        .expect("whir-p3 dependency in Cargo.toml");
    let revision = dependency
        .split_once("rev = \"")
        .expect("whir-p3 must use an immutable Git revision")
        .1
        .split('"')
        .next()
        .expect("Git source revision");
    println!("cargo:rustc-env=SPARTAN_WHIR_EXPORT_WHIR_REVISION={revision}");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    let rustflags = std::env::var("RUSTFLAGS").unwrap_or_else(|_| {
        std::env::var("CARGO_ENCODED_RUSTFLAGS")
            .unwrap_or_default()
            .replace('\x1f', " ")
    });
    println!(
        "cargo:rustc-env=SPARTAN_WHIR_EXPORT_RUSTFLAGS={}",
        rustflags
    );
}
