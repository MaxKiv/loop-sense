fn main() {
    #[cfg(feature = "nidaq")]
    println!("cargo:rustc-link-search=native=vendor/nidaqmx/lib64/msvc");
    #[cfg(feature = "nidaq")]
    println!("cargo:rustc-link-lib=static=NiDAQmx");
    #[cfg(feature = "nidaq")]
    println!("cargo:rerun-if-changed=vendor/nidaqmx/lib64/msvc/NIDAQmx.lib");
}
