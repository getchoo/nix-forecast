{
  description = "Check the forecast for today's Nix builds";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs =
    {
      self,
      nixpkgs,
      nix-filter,
    }:
    let
      inherit (nixpkgs) lib;

      # We want to generate outputs for as many systems as possible,
      # even if we don't officially support or test for them
      allSystems = lib.systems.flakeExposed;

      # These are the systems we do officially support and test, though
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = lib.genAttrs allSystems;
      nixpkgsFor = nixpkgs.legacyPackages;
    in
    {
      checks = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          packages = self.packages.${system};
        in
        lib.optionalAttrs (lib.elem system supportedSystems) {
          version-test = packages.nix-forecast.tests.version;

          clippy = packages.nix-forecast.overrideAttrs (oldAttrs: {
            pname = "check-clippy";

            nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
              pkgs.clippy
              pkgs.clippy-sarif
              pkgs.sarif-fmt
            ];

            buildPhase = ''
              runHook preBuild
              cargo clippy \
                --all-features \
                --all-targets \
                --tests \
                --message-format=json \
              | clippy-sarif | tee $out | sarif-fmt
            '';

            dontInstall = true;
            doCheck = false;
            dontFixup = true;

            passthru = { };
            meta = { };
          });

          formatting =
            pkgs.runCommand "check-formatting"
              {
                nativeBuildInputs = [
                  pkgs.cargo
                  pkgs.nixfmt-rfc-style
                  pkgs.rustfmt
                ];
              }
              ''
                cd ${self}

                echo "Running cargo fmt"
                cargo fmt -- --check

                echo "Running nixfmt..."
                nixfmt --check  .

                touch $out
              '';
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
        in
        lib.optionalAttrs (lib.elem system supportedSystems) {
          default = pkgs.mkShell {
            packages = [
              # Rust tools
              pkgs.clippy
              pkgs.rust-analyzer
              pkgs.rustfmt

              # Nix tools
              self.formatter.${system}
              pkgs.nil
              pkgs.statix
            ];

            env = {
              RUST_SRC_PATH = toString pkgs.rustPlatform.rustLibSrc;
            };

            inputsFrom = [ self.packages.${system}.nix-forecast ];
          };
        }
      );

      formatter = forAllSystems (system: nixpkgsFor.${system}.nixfmt-rfc-style);

      # for CI
      legacyPackages = forAllSystems (
        system:
        lib.optionalAttrs (lib.elem system supportedSystems) (
          lib.mapAttrs' (name: lib.nameValuePair "check-${name}") self.checks.${system}
        )
      );

      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          nixForecastPackages = lib.makeScope pkgs.newScope (final: self.overlays.default final pkgs);
        in
        {
          inherit (nixForecastPackages) nix-forecast;
          default = self.packages.${system}.nix-forecast;
        }
      );

      overlays.default = final: _: {
        nix-forecast = final.callPackage ./nix/package.nix { inherit nix-filter self; };
      };
    };
}
