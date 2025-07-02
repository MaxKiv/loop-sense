use std::env;

fn main() {
    // env::var("TARGET") gives you the full triple (e.g. x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc).
    let target = env::var("TARGET").unwrap_or_default();

    // Link to vendored NIDAQmx lib
    if cfg!(feature = "nidaq") {
        if target.contains("windows-msvc") {
            println!("cargo:rustc-link-search=native=vendor/nidaqmx/lib64/msvc");
            println!("cargo:rustc-link-lib=static=NiDAQmx");
            println!("cargo:rerun-if-changed=vendor/nidaqmx/lib64/msvc/NIDAQmx.lib");
        } else if target.contains("linux") {
            println!("cargo:rustc-link-search=native=vendor/nidaqmx/lib64/gcc");
            println!("cargo:rustc-link-lib=dylib=nidaqmx");
            println!("cargo:rerun-if-changed=vendor/nidaqmx/lib64/gcc/libnidaqmx.so");
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../vendor/nidaqmx/lib64/gcc");
        }
    }
}
