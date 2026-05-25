fn main() {
    let target = "riscv32.json";
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", target);
}