# focal

focal is a cli / rofi menu for capturing and copying screenshots or videos on hyprland / niri / mango / sway.

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
- supports hyprland / niri / mango / sway
- OCR support to select text from captured image (CLI only)

## Installation

### NixOS
```nix
{
  inputs.focal.url = "github:iynaix/focal";
}
```

A [focal cachix](https://focal.cachix.org) is also available, providing prebuilt binaries. To use it, add the following to your configuration:
```nix
{
  nix.settings = {
    substituters = ["https://focal.cachix.org"];
    trusted-public-keys = ["focal.cachix.org-1:/YkOWkXNH2uK7TnskrVMvda8LyCe4iIbMM1sZN2AOXY="];
  };
}
```

> [!Warning]
> Overriding the `focal` input using a `inputs.nixpkgs.follows` invalidates the cache and will cause the package to be rebuilt.


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

> [!Note]
> `rofi` now has wayland support on `nixos-unstable`. To use `rofi-wayland` for `nixos-25.05`, you can use the following override:
> ```nix
> (inputs.focal.packages.${pkgs.system}.default.override { rofi = pkgs.rofi-wayland; })
> ```

### Arch Linux

Arch Linux users can install from the [AUR](https://aur.archlinux.org/) or [AUR-git](https://aur.archlinux.org/packages/focal).

```sh
# focal detects the window manager at runtime, so focal-hyprland works for all other supported WMs as well
paru -S focal-hyprland-git
```

## Usage

```console
$ focal --help
focal is a cli / rofi menu for capturing and copying screenshots or videos on hyprland / niri /sway.

Usage: focal image [OPTIONS] <--rofi|--area <AREA>|--selection|--monitor|--all> [FILE]
       focal video [OPTIONS] <--rofi|--area <AREA>|--selection|--monitor|--stop> [FILE]
       focal help [COMMAND]...

Options:
  -h, --help     Print help
  -V, --version  Print version

focal image:
Captures a screenshot.
  -a, --area <AREA>         Type of area to capture [aliases: capture] [possible values: monitor, selection, all]
      --selection
      --monitor
      --all
      --freeze              Freezes the screen before selecting an area.
  -t, --delay <DELAY>       Delay in seconds before capturing
  -s, --slurp <SLURP>       Options to pass to slurp
      --no-rounded-windows  Do not show rounded corners when capturing a window. (Hyprland only)
      --no-notify           Do not show notifications
      --no-save             Do not save the file permanently
      --rofi                Display rofi menu for selection options
      --no-icons            Do not show icons for rofi menu
      --theme <THEME>       Path to a rofi theme
  -e, --edit <COMMAND>      Edit screenshot using COMMAND
                            The image path will be passed as $IMAGE
      --ocr [<LANG>]        Runs OCR on the selected text
  -h, --help                Print help (see more with '--help')
  [FILE]                Files are created in XDG_PICTURES_DIR/Screenshots if not specified

focal video:
Captures a video.
  -a, --area <AREA>         Type of area to capture [aliases: capture] [possible values: monitor, selection]
      --selection
      --monitor
  -t, --delay <DELAY>       Delay in seconds before capturing
  -s, --slurp <SLURP>       Options to pass to slurp
      --no-rounded-windows  Do not show rounded corners when capturing a window. (Hyprland only)
      --no-notify           Do not show notifications
      --no-save             Do not save the file permanently
      --rofi                Display rofi menu for selection options
      --no-icons            Do not show icons for rofi menu
      --theme <THEME>       Path to a rofi theme
      --stop                Stops any previous video recordings
      --audio [<DEVICE>]    Capture video with audio, optionally specifying an audio device
      --duration <SECONDS>  Duration in seconds to record
  -h, --help                Print help (see more with '--help')
  [FILE]                Files are created in XDG_VIDEOS_DIR/Screencasts if not specified

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

For a **niri** keybinding:
```
Mod+backslash { spawn "focal" "image" "--area" "selection" }
```

For a **mango** keybinding:
```
bind=SUPER, backslash, exec, focal image --area selection
```

For a **sway** keybinding:
```
bindsym $mod+backslash exec "focal image --area selection"
```

### Optional Waybar Module

An optional `focal-waybar` script is available for [waybar](https://github.com/Alexays/Waybar) to indicate when a recording is in progress.

```console
$ focal-waybar --help
Updates waybar module with focal's recording status.

Usage: focal-waybar [OPTIONS]

Options:
      --recording <MESSAGE>  Message to display in waybar module when recording [default: REC]
      --stopped <MESSAGE>    Message to display in waybar module when not recording [default: ]
  -h, --help                 Print help
  -V, --version              Print version
```

Create a custom waybar module similar to the following:

```jsonc
{
  "custom/focal": {
    "exec": "focal-waybar --recording 'REC'",
    "format": "{}",
    // interval to poll for updated recording status
    "interval": 1,
    "on-click": "focal video --stop",
  },
}
```

focal video recordings can then be started / stopped using keybindings such as:

**hyprland**:
```
bind=$mainMod, backslash, exec, focal video --rofi --audio
```

**niri**:
```
Mod+backslash { spawn "focal" "video" "--rofi" "--audio" }
```

**sway**:
```
bindsym $mod+backslash exec "focal video --rofi --audio"
```

## Packaging

To build focal from source

- Build dependencies
    * Rust (cargo, rustc)
- Runtime dependencies
    * [grim](https://sr.ht/~emersion/grim/)
    * [slurp](https://github.com/emersion/slurp)
    * [hyprland](https://hyprland.org/)
    * [niri](https://github.com/YaLTeR/niri)
    * [mango](https://github.com/DreamMaoMao/mangowc)
    * [sway](https://swaywm.org/)
    * [rofi](https://github.com/davatorium/rofi)
    * [wl-clipboard](https://github.com/bugaevc/wl-clipboard)
    * [wf-recorder](https://github.com/ammen99/wf-recorder)
    * [ffmpeg](https://www.ffmpeg.org/)

## Hacking

Just use `nix develop`
