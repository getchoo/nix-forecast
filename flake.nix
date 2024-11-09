{
  description = "Check the forecast for today's Nix builds";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs =
    {
      self,
      nixpkgs,
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

          mkCheck =
            name: deps: script:
            pkgs.runCommand name { nativeBuildInputs = deps; } ''
              ${script}
              touch $out
            '';
        in
        lib.optionalAttrs (lib.elem system supportedSystems) {
          clippy = packages.nix-forecast.overrideAttrs (oldAttrs: {
            pname = "check-clippy";

            nativeBuildInputs = oldAttrs.nativeBuildInputs or [ ] ++ [
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
              runHook postBuild
            '';

            dontInstall = true;
            doCheck = false;
            dontFixup = true;

            passthru = { };
            meta = { };
          });

          rustfmt = mkCheck "check-cargo-fmt" [
            pkgs.cargo
            pkgs.rustfmt
          ] "cd ${self} && cargo fmt -- --check";

          actionlint = mkCheck "check-actionlint" [
            pkgs.actionlint
          ] "actionlint ${self}/.github/workflows/*";

          deadnix = mkCheck "check-deadnix" [ pkgs.deadnix ] "deadnix --fail ${self}";

          nixfmt = mkCheck "check-nixfmt" [ pkgs.nixfmt-rfc-style ] "nixfmt --check ${self}";

          statix = mkCheck "check-statix" [ pkgs.statix ] "statix check ${self}";
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

      legacyPackages = forAllSystems (system: {
        nix-forecast-debug = self.packages.${system}.nix-forecast.overrideAttrs (
          finalAttrs: _: {
            cargoBuildType = "debug";
            cargoCheckType = finalAttrs.cargoBuildType;
          }
        );
      });

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
        nix-forecast = final.callPackage ./nix/package.nix { };
      };
    };
}
