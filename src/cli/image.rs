use std::path::PathBuf;

use crate::cli::focal::{CommonArgs, RofiArgs};
use clap::{ArgGroup, Args, Subcommand, ValueEnum};

#[derive(Subcommand, ValueEnum, Debug, Clone)]
pub enum CaptureArea {
    Monitor,
    Selection,
    Window,
    All,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("area_shortcuts")
        .args(["area", "window", "selection", "monitor", "all"])
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
        long_help = "Shorthand for --area selection"
    )]
    pub selection: bool,

    #[arg(
        long,
        group = "area_shortcuts",
        help = "",
        long_help = "Shorthand for --area window"
    )]
    pub window: bool,

    #[arg(
        long,
        group = "area_shortcuts",
        help = "",
        long_help = "Shorthand for --area monitor"
    )]
    pub monitor: bool,

    #[arg(
        long,
        group = "area_shortcuts",
        hide = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default() == "niri",
        help = "",
        long_help = "Shorthand for --area all"
    )]
    pub all: bool,
}

impl AreaArgs {
    pub fn parse(&self) -> Option<CaptureArea> {
        if self.selection {
            Some(CaptureArea::Selection)
        } else if self.window {
            Some(CaptureArea::Window)
        } else if self.monitor {
            Some(CaptureArea::Monitor)
        } else if self.all {
            Some(CaptureArea::All)
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
        .args(["rofi", "area", "selection", "window", "monitor", "all"]),
))]
#[command(group(
    ArgGroup::new("freeze_mode")
        .required(false)
        .multiple(false)
        .args(["rofi", "area", "selection"]),
))]
pub struct ImageArgs {
    #[command(flatten)]
    pub area_args: AreaArgs,

    #[arg(
        long,
        action,
        help = "Freezes the screen before selecting an area.",
        hide = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default() == "niri",
    )]
    pub freeze: bool,

    #[command(flatten)]
    pub common_args: CommonArgs,

    #[command(flatten)]
    pub rofi_args: RofiArgs,

    #[arg(
        short,
        long,
        action,
        help = "Edit screenshot using COMMAND\nThe image path will be passed as $IMAGE",
        value_name = "COMMAND",
        conflicts_with = "ocr"
    )]
    pub edit: Option<String>,

    #[arg(
        long,
        num_args = 0..=1,
        value_name = "LANG",
        default_missing_value = "",
        action,
        help = "Runs OCR on the selected text",
        long_help = "Runs OCR on the selected text, defaulting to English\nSupported languages can be shown using 'tesseract --list-langs'",
        conflicts_with = "edit",
        hide = cfg!(not(feature = "ocr"))
    )]
    pub ocr: Option<String>,

    #[arg(
        name = "FILE",
        help = "Files are created in XDG_PICTURES_DIR/Screenshots if not specified"
    )]
    pub filename: Option<PathBuf>,
}

impl ImageArgs {
    pub fn required_programs(&self) -> Vec<&str> {
        let mut progs = vec!["grim"];

        if self.rofi_args.rofi {
            progs.push("rofi");
            progs.push("slurp");
        }

        if matches!(self.area_args.parse(), Some(CaptureArea::Selection)) {
            progs.push("slurp");
        }

        if self.ocr.is_some() {
            progs.push("tesseract");
        }

        progs
    }
}
