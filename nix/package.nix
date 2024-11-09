{
  lib,
  installShellFiles,
  makeBinaryWrapper,
  nix,
  rustPlatform,
  versionCheckHook,

  nix-filter,
  self,
}:

rustPlatform.buildRustPackage rec {
  pname = "nix-forecast";
  inherit (passthru.cargoTOML.package) version;

  src = nix-filter {
    root = self;
    include = [
      "Cargo.toml"
      "Cargo.lock"
      "build.rs"
      "src/"
    ];
  };

  cargoLock.lockFile = self + "/Cargo.lock";

  nativeBuildInputs = [
    installShellFiles
    makeBinaryWrapper
  ];

  doInstallCheck = true;
  nativeInstallCheckInputs = [ versionCheckHook ];

  postInstall = ''
    wrapProgram $out/bin/nix-forecast --suffix PATH : ${lib.makeBinPath [ nix ]}

    installShellCompletion --cmd nix-forecast \
      --bash completions/nix-forecast.bash \
    	--fish completions/nix-forecast.fish \
    	--zsh completions/_nix-forecast
  '';

  env = {
    COMPLETION_DIR = "completions";
  };

  passthru = {
    cargoTOML = lib.importTOML (self + "/Cargo.toml");
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
