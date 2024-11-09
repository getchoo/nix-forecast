{
  pkgs ? import nixpkgs {
    inherit system;
    config = { };
    overlays = [ ];
  },
  lib ? pkgs.lib,
  nixpkgs ? <nixpkgs>,
  system ? builtins.currentSystem,
}:

let
  nixForecastPackages = lib.makeScope pkgs.newScope (lib.flip (import ./overlay.nix) pkgs);
in
{
  inherit (nixForecastPackages) nix-forecast;
}
