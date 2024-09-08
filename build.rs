fn main() {
    // override with the version passed in from nix
    // https://github.com/rust-lang/cargo/issues/6583#issuecomment-1259871885
    if let Ok(val) = std::env::var("NIX_RELEASE_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={val}");
    }
    println!("cargo:rerun-if-env-changed=NIX_RELEASE_VERSION");
}
