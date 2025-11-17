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
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # NixOS inputs
    nixos-hardware.url = "github:nixos/nixos-hardware/master";
  };

  # Outputs this flake produces
  outputs = {
    self,
    nixpkgs,
    crane,
    fenix,
    flake-utils,
    rust-overlay,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      # Define cross compilation targets
      targets = [
        "x86_64-unknown-linux-gnu"
        "x86_64-unknown-linux-musl"
        "aarch64-unknown-linux-gnu"
        "aarch64-unknown-linux-musl"
      ];

      overlays = [(import rust-overlay)];

      # Function to get pkgs for a given host & target
      pkgsFor = {
        localSystem,
        target ? null,
      }: let
        crossSystem =
          if lib.hasSuffix "musl" target
          then {
            config = target;
            isStatic = true;
          }
          else {config = target;};
      in
        import nixpkgs ({
            inherit localSystem overlays;
          }
          // (
            if target != null
            then {inherit crossSystem;}
            else {}
          ));

      # Function to get a rust toolchain usable for all cross compilation targets for a given pkgs
      rustForTarget = pkgs:
        pkgs.rust-bin.stable.latest.default.override {
          targets = targets;
        };

      # Base pkgs for the host system
      pkgsHost = pkgsFor {localSystem = system;};
      inherit (pkgsHost) lib;

      # Collect all the source files that need to be compiled
      src = lib.cleanSourceWith {
        src = (crane.mkLib pkgsHost).path ./.;
      };

      # Common build args
      commonArgs = {
        inherit src;
        strictDeps = true;

        nativeBuildInputs =
          [
            # pkgs.cmake
            # pkgs.nodejs_24
          ]
          ++ lib.optionals pkgsHost.stdenv.isLinux [
            pkgsHost.pkg-config
            # pkgs.rustPlatform.bindgenHook
          ];

        buildInputs = [
          # pkgs.cargo-nextest
          # pkgs.openssl.dev
          # ]
          # ++ lib.optional pkgs.stdenv.isDarwin [
          #   pkgs.libiconv
          #   pkgs.iconv
          #   pkgs.cacert
          #   pkgs.curl
        ];

        LIBCLANG_PATH = "${pkgsHost.llvmPackages.libclang.lib}/lib";
      };

      # Standard cargo artifacts for the host system
      baseCraneLib = (crane.mkLib pkgsHost).overrideToolchain (_: rustForTarget pkgsHost);
      cargoArtifacts = baseCraneLib.buildDepsOnly commonArgs;

      # Function to create a package for a specific target
      makePackage = target: let
        crossPkgs = pkgsFor {
          localSystem = system;
          inherit target;
        };

        # Use the buildPackages toolchain for cross builds!
        craneLib = (crane.mkLib crossPkgs).overrideToolchain (_: rustForTarget crossPkgs.buildPackages);

        # Only use static linking for musl targets
        isMusl = lib.hasSuffix "musl" target;

        targetArgs =
          commonArgs
          // {
            inherit cargoArtifacts;
            doCheck = false;

            # Set rust target
            CARGO_BUILD_TARGET = target;

            # Set build inputs
            buildInputs =
              commonArgs.buildInputs;

            # Only enable static CRT for musl targets
            CARGO_BUILD_RUSTFLAGS = lib.optionalString isMusl "-C target-feature=+crt-static";
          };
      in
        craneLib.buildPackage targetArgs;

      # Build package for each target
      packages = builtins.listToAttrs (
        map
        (target: {
          name = builtins.replaceStrings ["-"] ["_"] target;
          value = makePackage target;
        })
        targets
      );

      hostPackage = baseCraneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          doCheck = false;
        });
    in
      with pkgsHost; {
        packages =
          packages
          // {
            default = hostPackage;
          };

        devShells = {
          # default = pkgsLocal.mkShell {
          #   LD_LIBRARY_PATH = pkgsLocal.lib.makeLibraryPath [pkgsLocal.stdenv.cc.cc];
          #   RUST_BACKTRACE = "full";
          #   buildInputs = with pkgsLocal; [
          #     nil
          #     alejandra
          #     toolchain
          #     rust-analyzer
          #     influxdb3
          #     git-lfs
          #   ];
          # };
          default = baseCraneLib.devShell (commonArgs
            // {
              inputsFrom = [hostPackage];
              shellHook = ''
                echo "HHH Development Environment"
                echo "==========================="
                echo "Rust version: $(rustc --version)"
                echo "Cargo version: $(cargo --version)"
                alias c=cargo
                alias j=just
                alias nv=nvim
                alias n=nvim
                alias cr="cargo run"
                alias cb="cargo build"
                alias gs="git status"
                alias gp="git push"
                alias gpf="git push --force-with-lease"
                alias ga="git add ."
                export DYLD_LIBRARY_PATH="$(rustc --print sysroot)/lib:$DYLD_LIBRARY_PATH"
                export RUST_SRC_PATH="$(rustc --print sysroot)/lib/rustlib/src/rust/src"
              '';
            });
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
          loopSensePackage = self.packages.aarch64-linux.aarch64_unknown_linux_gnu;
          inherit inputs;
        };
        modules = [
          ./nixos/rpi3
        ];
      };

      # NixOS configuration for rpi4
      nixosConfigurations.rpi4 = nixpkgs.lib.nixosSystem {
        system = "aarch64-linux";
        specialArgs = {
          hostname = "rpi4";
          username = "max";
          sshPublicKeys = import ./nixos/resources/ssh_public_keys.nix;
          composePath = ./compose.yaml;
          snapshotPath = ./snapshot;
          resourcePath = ./nixos/resources;
          loopSensePackage = self.packages.aarch64-linux.aarch64_unknown_linux_gnu;
          inherit inputs;
        };
        modules = [
          ./nixos/rpi4
        ];
      };
    };
}
