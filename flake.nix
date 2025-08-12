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
                grim
                rofi-wayland
                slurp
                tesseract
                hyprpicker
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
            no-waybar = focal.override { focalWaybar = false; };
            focal-hyprland = focal.override { backend = "hyprland"; };
            focal-sway = focal.override { backend = "sway"; };
            focal-niri = focal.override { backend = "niri"; };
            # placeholder for future, not implemented!
            focal-mango = focal.override { backend = "mango"; };
            focal-image = focal.override { video = false; };
          };
        };
    };
}
