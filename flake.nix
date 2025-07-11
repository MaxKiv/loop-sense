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

    # NixOS inputs
    nixos-hardware.url = "github:nixos/nixos-hardware/master";
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
          sha256 = "sha256-iia8FkmVjcS5deG61FHlPDH/8Mh35VCsThCCgqRSJ2A=";
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
            ZIG_GLOBAL_CACHE_DIR = "$TMPDIR/.zig-cache";
            XDG_CACHE_HOME = "$TMPDIR/.zig-cache";
            CC = "${pkgs.zig}/bin/zig cc";
            AR = "${pkgs.zig}/bin/zig ar";
            CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "${pkgs.zig}/bin/zig cc";
            CC_x86_64_pc_windows_msvc = "${pkgs.zig}/bin/zig cc -target x86_64-windows-msvc";
            AR_x86_64_pc_windows_msvc = "${pkgs.zig}/bin/zig ar";
            # Ensure ring can detect the compiler
            RING_PREGENERATE_ASM = "1";
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
          # Clean up any existing pregenerated directories before build
          preBuild =
            ''
              find . -name "pregenerated" -type d -exec rm -rf {} + 2>/dev/null || true
              # Create cache directory if using Zig
            ''
            + (
              if target == "x86_64-pc-windows-msvc"
              then ''
                mkdir -p $TMPDIR/.zig-cache
                chmod 755 $TMPDIR/.zig-cache
              ''
              else ""
            );
        };
    in {
      # Development shells provided by this flake, to use:
      # nix develop .#default
      devShell = pkgs.mkShell {
        # Required by the nidaqmx lib
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
          pkgs.stdenv.cc.cc
        ];
        RUST_BACKTRACE = "full";

        buildInputs = with pkgs; [
          zig_0_13 # zig toolchain, used to compile Windows MSVC binaries
          nil # Nix LSP
          alejandra # Nix Formatter
          toolchain # Our Rust toolchain
          rust-analyzer # Rust LSP

          cargo-xwin # easy windows x-compilation

          influxdb3 # Timeseries Database
        ];
      };

      # Build outputs of this flake, accessible using nix build .#{output}
      packages = {
        default = buildForTarget "x86_64-unknown-linux-gnu" "sim";
        nidaq = buildForTarget "x86_64-unknown-linux-gnu" "nidaq";
        windows-gnu = buildForTarget "x86_64-pc-windows-gnu" "nidaq";
        windows-msvc = buildForTarget "x86_64-pc-windows-msvc" "nidaq";
      };

      # formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.alejandra;

      # Nixos Configuration outputs of this flake, accesible using nixos-rebuild .#{output}
      nixosConfigurations = {
        "rpi3" = let
          hostname = "rpi3";
          username = "max";
          system = "aarch64-linux";
        in
          nixpkgs.lib.nixosSystem {
            specialArgs =
              {
                sshPublicKeys = import ./nixos/resources/ssh_public_keys.nix;
                inherit system hostname username inputs;
              }
              // inputs;
            modules = [
              ./nixos/rpi3
            ];
          };
      };
    });
}
