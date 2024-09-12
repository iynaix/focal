use std::path::PathBuf;

use crate::{image::ImageArgs, rofi::RofiArgs, video::VideoArgs};
use clap::{ArgGroup, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};

#[derive(Subcommand, ValueEnum, Debug, Clone)]
pub enum CaptureArea {
    Monitor,
    Selection,
    All,
}

#[derive(Subcommand, ValueEnum, Debug, Clone)]
pub enum ShellCompletion {
    Bash,
    Zsh,
    Fish,
}

#[derive(Parser, Debug)]
#[command(
    name = "focal",
    about = "focal is a rofi menu for capturing and copying screenshots or videos on hyprland / sway.",
    author,
    version = env!("CARGO_PKG_VERSION")
)]
#[command(group(
    ArgGroup::new("mode")
        .multiple(false)
        .args(["edit", "ocr", "video"]),
))]
#[command(group(
    ArgGroup::new("required_operation")
    .required(true)
    .multiple(false)
    .args(["area", "rofi"]),
))]
pub struct Cli {
    #[arg(
        short,
        long,
        visible_alias = "capture",
        value_enum,
        help = "Type of area to capture"
    )]
    pub area: Option<CaptureArea>,

    #[arg(short = 't', long, help = "Delay in seconds before capturing")]
    pub delay: Option<u64>, // sleep uses u64

    #[arg(short, long, help = "Options to pass to slurp")]
    pub slurp: Option<String>,

    #[arg(long, action, help = "Do not show notifications")]
    pub no_notify: bool,

    #[arg(long, action, help = "Do not save the file permanently")]
    pub no_save: bool,

    #[arg(
        long,
        value_enum,
        help = "Type of shell completion to generate",
        hide = true,
        exclusive = true
    )]
    pub generate: Option<ShellCompletion>,

    #[arg(
        name = "FILE",
        long_help = "Files are created in XDG_PICTURES_DIR/Screenshots or XDG_VIDEOS_DIR/Screencasts if not specified"
    )]
    pub filename: Option<PathBuf>,

    #[command(flatten)]
    pub rofi_args: RofiArgs,

    #[command(flatten)]
    pub image_args: ImageArgs,

    #[command(flatten)]
    pub video_args: VideoArgs,
}

pub fn generate_completions(shell_completion: &ShellCompletion) {
    let mut cmd = Cli::command();

    match shell_completion {
        ShellCompletion::Bash => generate(Shell::Bash, &mut cmd, "focal", &mut std::io::stdout()),
        ShellCompletion::Zsh => generate(Shell::Zsh, &mut cmd, "focal", &mut std::io::stdout()),
        ShellCompletion::Fish => generate(Shell::Fish, &mut cmd, "focal", &mut std::io::stdout()),
    }
}

// write tests for nixos
#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    fn assert_cmd(cmd: &str, err_kind: ErrorKind, msg: &str) {
        let res = Cli::try_parse_from(cmd.split_whitespace());
        assert!(res.is_err());
        assert_eq!(res.expect_err("").kind(), err_kind, "{msg}");
    }

    #[test]
    fn test_exclusive_args() {
        assert_cmd(
            "focal --video --area monitor --edit gimp",
            ErrorKind::ArgumentConflict,
            "--video and --area are exclusive",
        );

        assert_cmd(
            "focal --area monitor --ocr --edit gimp",
            ErrorKind::ArgumentConflict,
            "--ocr and --edit are exclusive",
        );

        assert_cmd(
            "focal --video",
            ErrorKind::MissingRequiredArgument,
            "--area or --rofi is required",
        );

        assert_cmd(
            "focal --audio",
            ErrorKind::MissingRequiredArgument,
            "--video is required for --audio",
        );
    }
}
