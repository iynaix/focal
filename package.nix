{
  version,
  lib,
  installShellFiles,
  rustPlatform,
  makeWrapper,
  ffmpeg,
  grim,
  procps,
  rofi-wayland,
  slurp,
  tesseract,
  hyprpicker,
  wf-recorder,
  wl-clipboard,
  xdg-utils,
  hyprland,
  sway,
  backend ? "hyprland",
  ocr ? true,
  video ? true,
  focalWaybar ? true,
}:
assert lib.assertOneOf "backend" backend [
  "hyprland"
  "sway"
];
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

  cargoBuildFlags =
    [
      "--no-default-features"
      "--features"
      backend
    ]
    ++ lib.optionals video [
      "--features"
      "video"
    ]
    ++ lib.optionals ocr [
      "--features"
      "ocr"
    ]
    ++ lib.optionals focalWaybar [
      "--features"
      "waybar"
    ];

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
      binaries =
        [
          grim
          procps
          rofi-wayland
          slurp
          hyprpicker
          wl-clipboard
          xdg-utils
        ]
        ++ lib.optionals (backend == "hyprland") [ hyprland ]
        ++ lib.optionals (backend == "sway") [ sway ]
        ++ lib.optionals video [
          ffmpeg
          wf-recorder
        ]
        ++ lib.optionals ocr [ tesseract ];
    in
    "wrapProgram $out/bin/focal --prefix PATH : ${lib.makeBinPath binaries}";

  meta = with lib; {
    description = "Focal captures screenshots / videos using rofi, with clipboard support on hyprland";
    mainProgram = "focal";
    homepage = "https://github.com/iynaix/focal";
    license = licenses.mit;
    maintainers = [ maintainers.iynaix ];
  };
}
