fn main() {
    #[cfg(feature = "test-layers")]
    println!("cargo::rustc-env=RUST_LOG=trace");
}
