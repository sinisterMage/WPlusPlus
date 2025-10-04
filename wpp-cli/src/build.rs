fn main() {
    // Make absolutely sure Polly is never linked.
    println!("cargo:rustc-env=LLVM_SYS_NO_POLLY=1");
    // Some old llvm-sys versions ignore that, so we add redundancy:
    println!("cargo:rustc-env=LLVM_SYS_150_NO_POLLY=1");
}
