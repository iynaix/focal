{
  version,
  lib,
  installShellFiles,
  rustPlatform,
  makeWrapper,
  ffmpeg,
  grim,
  procps,
  rofi,
  slurp,
  tesseract,
  hyprpicker,
  wf-recorder,
  wl-clipboard,
  wlr-randr,
  xdg-utils,
  ocr ? true,
  video ? true,
  focalWaybar ? true,
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
  buildFeatures =
    lib.optionals video [ "video" ]
    ++ lib.optionals ocr [ "ocr" ]
    ++ lib.optionals focalWaybar [ "waybar" ];

  nativeBuildInputs = [
    installShellFiles
    makeWrapper
  ];

  postInstall =
    let
      bins = [ "focal" ] ++ lib.optionals focalWaybar [ "focal-waybar" ];
    in
    ''
      for cmd in ${lib.concatStringsSep " " bins}; do
        installShellCompletion --cmd $cmd \
          --bash <($out/bin/$cmd generate bash) \
          --fish <($out/bin/$cmd generate fish) \
          --zsh <($out/bin/$cmd generate zsh)
      done

      installManPage target/man/*
    '';

  postFixup =
    let
      binaries = [
        grim
        procps
        rofi
        slurp
        hyprpicker
        wl-clipboard
        wlr-randr
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
    description = "Focal captures screenshots / videos using rofi, with clipboard support on hyprland / niri / mango / sway";
    mainProgram = "focal";
    homepage = "https://github.com/iynaix/focal";
    license = licenses.mit;
    maintainers = [ maintainers.iynaix ];
  };
}
