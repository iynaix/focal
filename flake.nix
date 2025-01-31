{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default-linux";
    devenv.url = "github:cachix/devenv";
  };

  outputs =
    inputs@{
      devenv,
      flake-parts,
      nixpkgs,
      self,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ devenv.flakeModule ];
      systems = import inputs.systems;

      perSystem =
        { pkgs, ... }:
        {
          devShells = {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules = [
                {
                  # https://devenv.sh/reference/options/
                  dotenv.disableHint = true;

                  packages = with pkgs; [
                    cargo-edit
                    grim
                    hyprland
                    rofi-wayland
                    slurp
                    sway
                    tesseract
                    hyprpicker
                    wl-clipboard
                    xdg-utils # xdg-open
                  ];

                  languages.rust.enable = true;
                }
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
            focal-image = focal.override { video = false; };
          };
        };
    };

  nixConfig = {
    extra-substituters = [ "https://focal.cachix.org" ];
    extra-trusted-public-keys = [ "focal.cachix.org-1:/YkOWkXNH2uK7TnskrVMvda8LyCe4iIbMM1sZN2AOXY=" ];
  };
}
