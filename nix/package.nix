{
  lib,
  installShellFiles,
  makeBinaryWrapper,
  nix,
  rustPlatform,
  versionCheckHook,
}:

let
  fs = lib.fileset;
in
rustPlatform.buildRustPackage rec {
  pname = "nix-forecast";
  inherit (passthru.cargoTOML.package) version;

  src = fs.toSource {
    root = ../.;
    fileset = fs.intersection (fs.gitTracked ../.) (
      fs.unions [
        ../Cargo.lock
        ../Cargo.toml
        ../build.rs
        ../src
      ]
    );
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    installShellFiles
    makeBinaryWrapper
  ];

  doInstallCheck = true;
  nativeInstallCheckInputs = [ versionCheckHook ];

  # NOTE: Yes, we specifically need Nix. Lix does not have the newer
  # `path-info --json` output used internally
  postInstall = ''
    wrapProgram $out/bin/nix-forecast --prefix PATH : ${lib.makeBinPath [ nix ]}

    installShellCompletion --cmd nix-forecast \
      --bash completions/nix-forecast.bash \
      --fish completions/nix-forecast.fish \
      --zsh completions/_nix-forecast
  '';

  env = {
    COMPLETION_DIR = "completions";
  };

  passthru = {
    cargoTOML = lib.importTOML ../Cargo.toml;
  };

  meta = {
    description = "Check the forecast for today's Nix builds";
    homepage = "https://github.com/getchoo/nix-forecast";
    changelog = "https://github.com/getchoo/nix-forecast/releases/tag/${version}";
    license = lib.licenses.mpl20;
    maintainers = with lib.maintainers; [ getchoo ];
    mainProgram = "nix-forecast";
  };
}
