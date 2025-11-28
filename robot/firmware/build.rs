// This file was automatically generated.

fn main() {
    println!("cargo:linker=flip-link");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:llvm-args=--inline-threshold=5");
}
