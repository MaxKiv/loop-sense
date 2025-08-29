{
  description = "Nix flake for Holland Hybrid Heart";

  # Dependencies of this flake
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
  } @ inputs: let
    # Function to build our rust application for a given target architecture
    buildForTarget = localSystem: targetSystem: features: let
      pkgs = import nixpkgs {system = localSystem;};
      pkgsCross = import nixpkgs { system = targetSystem; };
      toolchain = with fenix.packages.${localSystem};
        fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-+9FmLhAOezBZCOziO0Qct1NOrfpjNsXxc/8I0c7BdKE=";
        };
      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
    in
      craneLib.buildPackage {
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            !pkgs.lib.hasSuffix "target" path
            && !pkgs.lib.hasInfix "/.git/" path;
        };

        strictDeps = true;
        doCheck = false;
        cargoExtraArgs = "--locked --features=" + features;
        CARGO_BUILD_TARGET = targetSystem;
        TARGET_CC = "${pkgsCross.stdenv.cc.targetPrefix}cc";

        env = pkgs.lib.optionalAttrs (targetSystem == "x86_64-pc-windows-msvc") {
          ZIG_GLOBAL_CACHE_DIR = "$TMPDIR/.zig-cache";
          XDG_CACHE_HOME = "$TMPDIR/.zig-cache";
          CC = "${pkgs.zig}/bin/zig cc";
          AR = "${pkgs.zig}/bin/zig ar";
          CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "${pkgs.zig}/bin/zig cc";
          CC_x86_64_pc_windows_msvc = "${pkgs.zig}/bin/zig cc -target x86_64-windows-msvc";
          AR_x86_64_pc_windows_msvc = "${pkgs.zig}/bin/zig ar";
          RING_PREGENERATE_ASM = "1";
        };

        OPENSSL_DIR = "${pkgs.openssl.dev}";
        OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

        depsBuildBuild = with pkgs;
          lib.optionals (targetSystem == "x86_64-pc-windows-gnu") [
            pkgsCross.mingwW64.stdenv.cc
            pkgsCross.mingwW64.windows.pthreads
          ];

        preBuild =
          ''
            find . -name "pregenerated" -type d -exec rm -rf {} + 2>/dev/null || true
          ''
          + (
            if targetSystem == "x86_64-pc-windows-msvc"
            then ''
              mkdir -p $TMPDIR/.zig-cache
              chmod 755 $TMPDIR/.zig-cache
            ''
            else ""
          );
      };
  in
    flake-utils.lib.eachDefaultSystem (localSystem:
      # Dev shells and packages for each supported system
      let
        pkgs = import nixpkgs {system = localSystem;};
        toolchain = with fenix.packages.${localSystem};
          fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-+9FmLhAOezBZCOziO0Qct1NOrfpjNsXxc/8I0c7BdKE=";
          };
      in {
        packages = {
          default = buildForTarget localSystem "x86_64-unknown-linux-gnu" "";
          rpi3 = buildForTarget localSystem "aarch64-unknown-linux-gnu" "";
        };

        devShells = {
          default = pkgs.mkShell {
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.stdenv.cc.cc];
            RUST_BACKTRACE = "full";
            buildInputs = with pkgs; [
              zig_0_13
              nil
              alejandra
              toolchain
              rust-analyzer
              cargo-xwin
              influxdb3
              git-lfs
            ];
          };
        };
      })
    // {
      # NixOS configuration for rpi3
      nixosConfigurations.rpi3 = nixpkgs.lib.nixosSystem {
        system = "aarch64-linux";
        specialArgs = {
          hostname = "rpi3";
          username = "max";
          sshPublicKeys = import ./nixos/resources/ssh_public_keys.nix;
          composePath = ./compose.yaml;
          snapshotPath = ./snapshot;
          resourcePath = ./nixos/resources;
          inherit inputs;
        };
        modules = [
          ./nixos/rpi3
        ];
      };
    };
}
