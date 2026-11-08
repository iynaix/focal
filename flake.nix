{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs =
    inputs@{
      flake-parts,
      nixpkgs,
      self,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      perSystem =
        { pkgs, ... }:
        {
          devShells = {
            default = pkgs.mkShell {
              packages = with pkgs; [
                cargo-edit
                rofi
                slurp
                tesseract
                libgbm
                libGL
                wayland
                wl-clipboard
                xdg-utils # xdg-open
              ];

              env = {
                # Required by rust-analyzer
                RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
              };

              nativeBuildInputs = with pkgs; [
                cargo
                rustc
                rust-analyzer
                rustfmt
                clippy
                pkg-config
              ];
            };
          };

          packages = rec {
            focal = pkgs.callPackage ./package.nix {
              version =
                if self ? "shortRev" then
                  self.shortRev
                else
                  nixpkgs.lib.replaceStrings [ "-dirty" ] [ "" ] self.dirtyShortRev;
            };
            default = focal;
            no-ocr = focal.override { ocr = false; };
            focal-image = focal.override { video = false; };
          };
        };
    };
}
