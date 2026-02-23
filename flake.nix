{
  description = "Nix flake for Holland Hybrid Heart";

  # Dependencies of this flake
  inputs = {
    your-nixos-flake.url = "github:maxkiv/nix";
    nixpkgs.follows = "your-nixos-flake/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    # fenix = {
    #   url = "github:nix-community/fenix";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
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
    flake-utils,
    rust-overlay,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      localSystem: let
        pkgs = import nixpkgs {
          inherit localSystem;
          overlays = [(import rust-overlay)];
          config = {
            allowUnfree = true;
          };
        };

        pkgsCrossAarch64 = import nixpkgs {
          inherit localSystem;
          crossSystem = {
            config = "aarch64-unknown-linux-gnu";
            rust.rustcTarget = "aarch64-unknown-linux-gnu";
          };
          overlays = [(import rust-overlay)];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);
        craneLibAarch64 = (crane.mkLib pkgsCrossAarch64).overrideToolchain (p: p.rust-bin.stable.latest.default);

        # Note: we have to use the `callPackage` approach here so that Nix
        # can "splice" the packages in such a way that dependencies are
        # compiled for the appropriate targets. If we did not do this, we
        # would have to manually specify things like
        # `nativeBuildInputs = with pkgs.pkgsBuildHost; [ someDep ];` or
        # `buildInputs = with pkgs.pkgsHostHost; [ anotherDep ];`.
        #
        # Normally you can stick this function into its own file and pass
        # its path to `callPackage`.
        crateExpression = {
          craneLib,
          pkgs,
          # openssl,
          # libiconv,
          # lib,
          # pkg-config,
          # stdenv,
        }:
          craneLib.buildPackage {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;

            # Dependencies which need to be build for the current platform
            # on which we are doing the cross compilation. In this case,
            # pkg-config needs to run on the build platform so that the build
            # script can find the location of openssl. Note that we don't
            # need to specify the rustToolchain here since it was already
            # overridden above.
            nativeBuildInputs = with pkgs;
              [
                pkg-config
              ]
              ++ lib.optionals stdenv.buildPlatform.isDarwin [
                libiconv
              ];

            # Dependencies which need to be built for the platform on which
            # the binary will run. In this case, we need to compile openssl
            # so that it can be linked with our executable.
            buildInputs = with pkgs; [
              # Add additional build inputs here
              openssl.dev
            ];
          };

        # Assuming the above expression was in a file called myCrate.nix
        # this would be defined as:
        # my-crate = pkgs.callPackage ./myCrate.nix { };
        my-crate = pkgs.callPackage crateExpression {
          inherit craneLib pkgs;
        };
        my-crate-aarch64 = pkgs.callPackage crateExpression {
          craneLib = craneLibAarch64;
          pkgs = pkgsCrossAarch64;
        };
      in {
        checks = {
          inherit my-crate;
        };

        packages.default = my-crate;
        packages.aarch64 = craneLibAarch64.buildPackage {
          src = craneLibAarch64.cleanCargoSource ./.;
          strictDeps = true;

          # Dependencies which need to be build for the current platform
          # on which we are doing the cross compilation. In this case,
          # pkg-config needs to run on the build platform so that the build
          # script can find the location of openssl. Note that we don't
          # need to specify the rustToolchain here since it was already
          # overridden above.
          nativeBuildInputs = with pkgsCrossAarch64;
            [
              pkg-config
            ]
            ++ lib.optionals stdenv.buildPlatform.isDarwin [
              libiconv
            ];

          # Dependencies which need to be built for the platform on which
          # the binary will run. In this case, we need to compile openssl
          # so that it can be linked with our executable.
          buildInputs = with pkgsCrossAarch64; [
            # Add additional build inputs here
            openssl.dev
          ];
          # CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";

          # patch after build
          # postInstall = ''
          #   patchelf \
          #     --set-interpreter /lib/ld-linux-aarch64.so.1 \
          #     $out/bin/loop_sense
          # '';
        };

        apps.default = flake-utils.lib.mkApp {
          drv = pkgs.writeScriptBin "loop-sense" ''
            ${pkgs.pkgsBuildBuild.qemu}/bin/qemu-aarch64 ${my-crate}/bin/loop-sense
          '';
        };
        apps.test-aarch64 = flake-utils.lib.mkApp {
          drv = pkgs.writeScriptBin "loop-sense" ''
            ${pkgs.pkgsBuildBuild.qemu}/bin/qemu-aarch64 ${my-crate-aarch64}/bin/loop-sense
          '';
        };

        devShells.default = craneLib.devShell {
          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEV_URL = "http://localhost:3000";

          # Automatically inherit any build inputs from `my-crate`
          inputsFrom = [my-crate];

          # Extra inputs (only used for interactive development)
          # can be added here; cargo and rustc are provided by default.
          packages = [
            # pkgs.cargo-audit
            # pkgs.cargo-watch
          ];
        };

        devShells.aarch64 = craneLibAarch64.devShell {
          # Automatically inherit any build inputs from the aarch64 build
          inputsFrom = [my-crate-aarch64];
        };
      }
    )
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
          loopSensePackage = self.packages.aarch64;
          inherit inputs;
        };
        moules = [
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
          composePath = ./nixos/rpi4/compose.yaml;
          loopSensePackage = self.packages.aarch64;
          inherit inputs;
        };
        modules = [
          ./nixos/rpi4
        ];
      };
    };
}
