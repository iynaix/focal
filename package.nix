{
  version,
  lib,
  installShellFiles,
  rustPlatform,
  makeWrapper,
  ffmpeg,
  grim,
  procps,
  slurp,
  tesseract,
  hyprpicker,
  pkg-config,
  libgbm,
  libGL,
  wayland,
  wf-recorder,
  wl-clipboard,
  xdg-utils,
  ocr ? true,
  video ? true,
}:
rustPlatform.buildRustPackage {
  pname = "focal";

  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.difference ./. (
      # don't include in build
      lib.fileset.unions [
        ./README.md
        ./LICENSE
        # ./PKGBUILD
      ]
    );
  };

  inherit version;

  # inject version from nix into the build
  env.NIX_RELEASE_VERSION = version;

  cargoLock.lockFile = ./Cargo.lock;

  buildNoDefaultFeatures = true;
  buildFeatures = lib.optionals video [ "video" ] ++ lib.optionals ocr [ "ocr" ];

  nativeBuildInputs = [
    installShellFiles
    makeWrapper
    pkg-config
  ];

  buildInputs = [
    libgbm
    libGL
    wayland
  ];

  postInstall = ''
    installShellCompletion --cmd focal \
      --bash <($out/bin/focal generate bash) \
      --fish <($out/bin/focal generate fish) \
      --zsh <($out/bin/focal generate zsh)

    installManPage target/man/*
  '';

  postFixup =
    let
      binaries = [
        grim
        procps
        slurp
        hyprpicker
        wl-clipboard
        xdg-utils
      ]
      ++ lib.optionals video [
        ffmpeg
        wf-recorder
      ]
      ++ lib.optionals ocr [ tesseract ];
    in
    "wrapProgram $out/bin/focal --prefix PATH : ${lib.makeBinPath binaries}";

  meta = with lib; {
    description = "Focal is a cli / graphical menu for capturing and copying screenshots or videos on hyprland / niri / umbriel / mango / sway.";
    mainProgram = "focal";
    homepage = "https://github.com/iynaix/focal";
    license = licenses.mit;
    maintainers = [ maintainers.iynaix ];
  };
}
