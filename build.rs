fn main() {
    #[cfg(feature = "nidaq")]
    println!("cargo:rustc-link-lib=NiDAQmx");
    #[cfg(feature = "nidaq")]
    println!(
        "cargo:rustc-link-search=native=C:/Program Files (x86)/National Instruments/NI-DAQ/DAQmx ANSI C Dev/lib/msvc"
    );
}
