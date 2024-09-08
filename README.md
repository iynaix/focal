# focal

focal captures screenshots / videos using rofi, with clipboard support on hyprland.

<img src="https://i.imgur.com/zsylLiC.png" alt="main menu" height="240" />
<img src="https://i.imgur.com/AipxMmf.png" alt="delay menu" height="240" />
<img src="https://i.imgur.com/aVMtOIe.png" alt="selection" height="240" />
<br/>
<em>Wallpaper made by the awesome <a href="https://www.pixiv.net/en/users/2993192">Rosuuri</a></em>

## Features

- rofi menu to select area or window to capture / capture the entire screen
- rofi menu to select delay before capture
- image / video is automatically copied to clipboard, ready for pasting into other programs
- notifications when screenshots are captured
- all options are also avaiable via a CLI
- OCR support to select text from captured image (CLI only)

## Installation

```nix
{
  inputs.focal = {
    url = "github:iynaix/focal";
    inputs.nixpkgs.follows = "nixpkgs"; # override this repo's nixpkgs snapshot
  };
}
```

Then, include it in your `environment.systemPackages` or `home.packages` by referencing the input:
```nix
inputs.focal.packages.${pkgs.system}.default
```

Alternatively, it can also be run directly:

```sh
nix run github:iynaix/focal
```

OCR support can be optionally disabled through the use of an override:
```nix
(inputs.focal.packages.${pkgs.system}.default.override { ocr = false; })
```

## Usage

```console
$ focal --help
Focal captures screenshots / videos using rofi, with clipboard support on hyprland

Usage: focal [OPTIONS] [FILE]

Arguments:
  [FILE]
          files are created in XDG_PICTURES_DIR/Screenshots or XDG_VIDEOS_DIR/Screencasts if not specified

Options:
      --rofi
          display rofi menu for options

      --theme <THEME>
          use rofi theme

      --area <AREA>
          type of area to capture

          [possible values: monitor, selection, all]

      --delay <DELAY>
          delay in seconds before capturing

      --no-notify
          do not show notifications

      --no-save
          do not save the file permanently

      --video
          record video instead of screenshots

      --audio
          capture video with audio

      --edit <PROGRAM>
          edit screenshot using PROGRAM

      --ocr [<LANG>]
          runs OCR on the selected text, defaulting to English, supported languages can be shown using 'tesseract --list-langs'

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Packaging

To build focal from source

- Build dependencies
    * Rust (cargo, rustc)
- Runtime dependencies
    * [grim](https://sr.ht/~emersion/grim/)
    * [libnotify](https://gitlab.gnome.org/GNOME/libnotify)
    * [slurp](https://github.com/emersion/slurp)
    * [hyprland](https://github.com/hyprwm/Hyprland)
    * [rofi-wayland](https://github.com/lbonn/rofi)
    * [wl-clipboard](https://github.com/bugaevc/wl-clipboard)
    * [wf-recorder](https://github.com/ammen99/wf-recorder)
    * [ffmpeg](https://www.ffmpeg.org/)

## Hacking

Just use `nix develop`