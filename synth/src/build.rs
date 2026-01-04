// TODO WTF???
#[allow(dead_code)]
fn main() {
    // Tell Cargo that this crate *depends* on the generated file
    println!(
        "cargo:rerun-if-changed={}/tables.rs",
        std::env::var("OUT_DIR").unwrap_or_default()
    );
}
