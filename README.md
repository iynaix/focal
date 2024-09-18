# focal

focal is a rofi menu for capturing and copying screenshots or videos on hyprland / sway.

<!-- 93859049_p0.webp -->
<img src="https://i.imgur.com/3DrXV0I.png" alt="main menu" width="49%" /> <img src="https://i.imgur.com/3kKoNJv.png" alt="delay menu" width="49%" />
<img src="https://i.imgur.com/5NXnkKm.png" alt="selection" width="49%" /> <img src="https://i.imgur.com/sm7PJgw.png" alt="selection" width="49%" />
<br/>
<em>Wallpaper made by the awesome <a href="https://www.pixiv.net/en/users/2993192">Rosuuri</a></em>

## Features

- rofi menu to select area / window / entire screen to capture
- rofi menu to select delay before capture
- image / video is automatically copied to clipboard, ready for pasting into other programs
- notifications that open captured file when clicked
- all options are also available via the CLI
- supports either hyprland or sway
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
# for hyprland
inputs.focal.packages.${pkgs.system}.default
# for sway
inputs.focal.packages.${pkgs.system}.focal-sway
```

Alternatively, it can also be run directly:

```sh
# for hyprland
nix run github:iynaix/focal
# for sway
nix run github:iynaix/focal#focal-sway
```

OCR support can be optionally disabled through the use of an override:
```nix
(inputs.focal.packages.${pkgs.system}.default.override { ocr = false; })
```

## Usage

```console
$ focal --help
focal is a rofi menu for capturing and copying screenshots or videos on hyprland / sway.

Usage: focal image [OPTIONS] <--rofi|--area <AREA>|--selection|--monitor|--all> [FILE]
       focal video [OPTIONS] <--rofi|--area <AREA>|--selection|--monitor|--stop> [FILE]
       focal help [COMMAND]...

Options:
  -h, --help     Print help
  -V, --version  Print version

focal image:
Captures a screenshot.
  -a, --area <AREA>     Type of area to capture [aliases: capture] [possible values: monitor, selection, all]
      --selection
      --monitor
      --all
  -t, --delay <DELAY>   Delay in seconds before capturing
  -s, --slurp <SLURP>   Options to pass to slurp
      --no-notify       Do not show notifications
      --no-save         Do not save the file permanently
      --rofi            Display rofi menu for selection options
      --no-icons        Do not show icons for rofi menu
      --theme <THEME>   Path to a rofi theme
  -e, --edit <COMMAND>  Edit screenshot using COMMAND
                        The image path will be passed as $IMAGE
      --ocr [<LANG>]    Runs OCR on the selected text
  -h, --help            Print help (see more with '--help')
  [FILE]            Files are created in XDG_PICTURES_DIR/Screenshots if not specified

focal video:
Captures a video.
  -a, --area <AREA>          Type of area to capture [aliases: capture] [possible values: monitor, selection]
      --selection
      --monitor
  -t, --delay <DELAY>        Delay in seconds before capturing
  -s, --slurp <SLURP>        Options to pass to slurp
      --no-notify            Do not show notifications
      --no-save              Do not save the file permanently
      --rofi                 Display rofi menu for selection options
      --no-icons             Do not show icons for rofi menu
      --theme <THEME>        Path to a rofi theme
      --stop                 Stops any previous video recordings
      --audio                Capture video with audio
      --duration <SECONDS>   Duration in seconds to record
  -h, --help                 Print help (see more with '--help')
  [FILE]                 Files are created in XDG_VIDEOS_DIR/Screencasts if not specified

focal help:
Print this message or the help of the given subcommand(s)
  [COMMAND]...  Print help for the subcommand(s)
```

> [!TIP]
> Invoking `focal video` a second time stops any currently recording videos.

Example usage as a **hyprland** keybinding:
```
bind=$mainMod, backslash, exec, focal image --area selection
```

Similarly, for a **sway** keybinding:
```
bindsym $mod+backslash exec "focal image --area selection"
```

### Optional Waybar Module

An optional `focal-waybar` script is available for [waybar](https://github.com/Alexays/Waybar) to indicate when a recording is in progress.

```console
$ focal-waybar --help
Updates waybar module with focal's recording status.

Usage: focal-waybar [OPTIONS] [FOCAL_ARGS]...

Arguments:
  [FOCAL_ARGS]...  Additional arguments to pass to 'focal video'

Options:
      --toggle               Start / stop focal recording
      --signal <N>           Signal number to update module (SIGRTMIN+N), default is 1 [default: 1]
      --recording <MESSAGE>  Message to display in waybar module when recording [default: REC]
      --stopped <MESSAGE>    Message to display in waybar module when not recording [default: ]
  -h, --help                 Print help
  -V, --version              Print version
```

Create a custom waybar module similar to the following:

```jsonc
{
  "custom/focal": {
    "exec": "focal-waybar",
    "format": "{}",
    "interval": "once",
    "on-click": "focal video --stop",
    "signal": 1
  },
}
```

focal video recordings can then be started / stopped using keybindings such as:

**hyprland**:
```
bind=$mainMod, backslash, exec, focal-waybar --toggle --signal 1 --recording 'REC' --rofi
```

**sway**:
```
bindsym $mod+backslash exec "focal-waybar --toggle --signal 1 --recording 'REC' --rofi"
```

## Packaging

To build focal from source

- Build dependencies
    * Rust (cargo, rustc)
- Runtime dependencies
    * [grim](https://sr.ht/~emersion/grim/)
    * [slurp](https://github.com/emersion/slurp)
    * [hyprland](https://hyprland.org/)
    * [sway](https://swaywm.org/)
    * [rofi-wayland](https://github.com/lbonn/rofi)
    * [wl-clipboard](https://github.com/bugaevc/wl-clipboard)
    * [wf-recorder](https://github.com/ammen99/wf-recorder)
    * [ffmpeg](https://www.ffmpeg.org/)

## Hacking

Just use `nix develop`