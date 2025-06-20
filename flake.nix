{
  description = "Rust dev env using nix";

  inputs = {
    your-nixos-flake.url = "github:maxkiv/nix";
    nixpkgs.follows = "your-nixos-flake/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  # Outputs this flake produces
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    crane,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = (import nixpkgs) {
        inherit system;
      };

      # Get a cross compilation toolchain from the rust-toolchain.toml
      toolchain = with fenix.packages.${system};
        fromToolchainFile {
          file = ./rust-toolchain.toml; # alternatively, dir = ./.;
          sha256 = "sha256-4RPRix7Kv4PT0n2YUOrpyVamDS007JGIOet8K29wEJg=";
          # sha256 = pkgs.lib.fakeSha256;
        };

      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

      buildForTarget = target: features:
        craneLib.buildPackage {
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              !pkgs.lib.hasSuffix "target" path
              && !pkgs.lib.hasInfix "/.git/" path;
          };

          strictDeps = true;
          doCheck = false;

          # Flags for cargo, set features here
          cargoExtraArgs = "--locked --features=" + features;

          # Define build target
          CARGO_BUILD_TARGET = target;

          # fixes issues related to libring
          # TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";
          TARGET_CC = "${pkgs.stdenv.cc}/bin/${pkgs.stdenv.cc.targetPrefix}cc";

          # Use Zig for MSVC builds
          env = pkgs.lib.optionalAttrs (target == "x86_64-pc-windows-msvc") {
            ZIG_GLOBAL_CACHE_DIR = ".zig-cache";
            XDG_CACHE_HOME = ".zig-cache";
            CC = "${pkgs.zig}/bin/zig";
            AR = "${pkgs.zig}/bin/zig ar";
            CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "${pkgs.zig}/bin/zig";
            CC_x86_64_pc_windows_msvc = "${pkgs.zig}/bin/zig cc -target x86_64-windows-msvc";
            AR_x86_64_pc_windows_msvc = "${pkgs.zig}/bin/zig ar";
            # RUSTFLAGS = "-L native=$NI_DAQMX_LIB_PATH";
          };

          #fixes issues related to openssl
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

          depsBuildBuild = with pkgs;
            lib.optionals (target == "x86_64-pc-windows-gnu") [
              pkgsCross.mingwW64.stdenv.cc
              pkgsCross.mingwW64.windows.pthreads
            ];

          # Link to vendored NI-DAQmx
          preBuild = with pkgs;
            lib.optionals (target == "x86_64-pc-windows-gnu")
            ''
              export NI_DAQMX_LIB_PATH=$PWD/vendor/nidaqmx/lib64/msvc
              export NI_DAQMX_INCLUDE_PATH=$PWD/vendor/nidaqmx/include
            '';
        };
    in {
      # Development shells provided by this flake, to use:
      # nix develop .#default
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          zig # zig toolchain, used to compile Windows MSVC binaries
          nil # Nix LSP
          alejandra # Nix Formatter
          toolchain # Our Rust toolchain
          rust-analyzer # Rust LSP

          influxdb3 # Timeseries Database
        ];
      };

      packages = {
        default = buildForTarget "x86_64-unknown-linux-gnu" "sim";
        windows-gnu = buildForTarget "x86_64-pc-windows-gnu" "nidaq";
        windows-msvc = buildForTarget "x86_64-pc-windows-msvc" "nidaq";
      };

      formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.alejandra;
    });
}
