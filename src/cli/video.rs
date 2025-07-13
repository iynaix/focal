use std::path::PathBuf;

use super::{CommonArgs, RofiArgs};
use clap::{ArgGroup, Args, Subcommand, ValueEnum};

#[derive(Subcommand, ValueEnum, Debug, Clone)]
pub enum CaptureArea {
    Monitor,
    Selection,
}

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("area_shortcuts")
        .args(["area", "selection", "monitor"])
        .multiple(false)
))]
pub struct AreaArgs {
    #[arg(
        short,
        long,
        visible_alias = "capture",
        value_enum,
        help = "Type of area to capture",
        long_help = "Type of area to capture\nShorthand aliases are also available"
    )]
    pub area: Option<CaptureArea>,

    #[arg(
        long,
        group = "area_shortcuts",
        help = "",
        long_help = "Shorthand for --area selection",
        hide = cfg!(feature = "niri")
    )]
    pub selection: bool,

    #[arg(
        long,
        group = "area_shortcuts",
        help = "",
        long_help = "Shorthand for --area monitor"
    )]
    pub monitor: bool,
}

impl AreaArgs {
    pub fn parse(&self) -> Option<CaptureArea> {
        if self.selection {
            Some(CaptureArea::Selection)
        } else if self.monitor {
            Some(CaptureArea::Monitor)
        } else {
            self.area.clone()
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("required_mode")
        .required(true)
        .multiple(false)
        .args(["rofi", "area", "selection", "monitor", "stop"]),
))]
pub struct VideoArgs {
    #[command(flatten)]
    pub area_args: AreaArgs,

    #[command(flatten)]
    pub common_args: CommonArgs,

    #[command(flatten)]
    pub rofi_args: RofiArgs,

    #[arg(long, action, help = "Stops any previous video recordings")]
    pub stop: bool,

    #[arg(
        long,
        num_args = 0..=1,
        value_name = "DEVICE",
        default_missing_value = "",
        help = "Capture video with audio, optionally specifying an audio device",
        long_help = "Capture video with audio, optionally specifying an audio device\nYou can find your device by running: pactl list sources | grep Name"
    )]
    pub audio: Option<String>,

    #[arg(
        long,
        value_name = "SECONDS",
        action,
        help = "Duration in seconds to record"
    )]
    pub duration: Option<u64>,

    #[arg(
        name = "FILE",
        help = "Files are created in XDG_VIDEOS_DIR/Screencasts if not specified"
    )]
    pub filename: Option<PathBuf>,
}

impl VideoArgs {
    pub fn required_programs(&self) -> Vec<&str> {
        let mut progs = vec!["wf-recorder", "pkill"];

        if self.rofi_args.rofi {
            progs.push("rofi");
            progs.push("slurp");
        }

        if matches!(self.area_args.parse(), Some(CaptureArea::Selection)) {
            progs.push("slurp");
        }

        progs
    }
}
