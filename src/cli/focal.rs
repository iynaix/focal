use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};

#[allow(clippy::module_name_repetitions)]
#[derive(Subcommand, Debug)]
pub enum FocalSubcommand {
    #[command(name = "image", about = "Captures a screenshot.")]
    Image(super::image::ImageArgs),

    #[cfg(feature = "video")]
    #[command(name = "video", about = "Captures a video.")]
    Video(super::video::VideoArgs),

    #[command(name = "generate", about = "Generate shell completions", hide = true)]
    Generate(GenerateArgs),
}

#[derive(Subcommand, ValueEnum, Debug, Clone)]
pub enum ShellCompletion {
    Bash,
    Zsh,
    Fish,
}

#[derive(Args, Debug)]
pub struct GenerateArgs {
    #[arg(value_enum, help = "Type of shell completion to generate")]
    pub shell: ShellCompletion,
}

#[derive(Args, Debug)]
pub struct CommonArgs {
    #[arg(short = 't', long, help = "Delay in seconds before capturing")]
    pub delay: Option<u64>, // sleep uses u64

    #[arg(short, long, help = "Options to pass to slurp")]
    pub slurp: Option<String>,

    // not available for sway
    #[arg(
        long,
        hide = cfg!(not(feature = "hyprland")),
        help = "Do not show rounded corners when capturing a window. (Hyprland only)"
    )]
    pub no_rounded_windows: bool,

    #[arg(long, action, help = "Do not show notifications")]
    pub no_notify: bool,

    #[arg(long, action, help = "Do not save the file permanently")]
    pub no_save: bool,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Args, Debug)]
pub struct RofiArgs {
    #[arg(long, action, help = "Display rofi menu for selection options")]
    pub rofi: bool,

    #[arg(long, action, help = "Do not show icons for rofi menu")]
    pub no_icons: bool,

    #[arg(long, action, help = "Path to a rofi theme")]
    pub theme: Option<PathBuf>,
}

#[derive(Parser, Debug)]
#[command(
    name = "focal",
    about = "focal is a cli / rofi menu for capturing and copying screenshots or videos on hyprland / sway.",
    author,
    version = env!("CARGO_PKG_VERSION"),
    infer_subcommands = true,
    flatten_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: FocalSubcommand,
}

pub fn generate_completions(
    progname: &str,
    cmd: &mut clap::Command,
    shell_completion: &ShellCompletion,
) {
    match shell_completion {
        ShellCompletion::Bash => generate(Shell::Bash, cmd, progname, &mut std::io::stdout()),
        ShellCompletion::Zsh => generate(Shell::Zsh, cmd, progname, &mut std::io::stdout()),
        ShellCompletion::Fish => generate(Shell::Fish, cmd, progname, &mut std::io::stdout()),
    }
}

// write tests for exclusive arguments
#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    fn assert_cmd(cmd: &str, err_kind: ErrorKind, msg: &str) {
        let args = Cli::try_parse_from(cmd.split_whitespace());
        assert!(args.is_err(), "{msg}");
        assert_eq!(args.expect_err("").kind(), err_kind, "{msg}");
    }

    #[test]
    fn test_exclusive_args() {
        assert_cmd(
            "focal video --rofi --area monitor",
            ErrorKind::ArgumentConflict,
            "--rofi and --area are exclusive",
        );

        assert_cmd(
            "focal image --rofi --area monitor",
            ErrorKind::ArgumentConflict,
            "--rofi and --area are exclusive",
        );

        assert_cmd(
            "focal image --area monitor --ocr --edit gimp",
            ErrorKind::ArgumentConflict,
            "--ocr and --edit are exclusive",
        );

        let res = Cli::try_parse_from("focal generate fish".split_whitespace());
        assert!(res.is_ok(), "generate should still work");
    }
}
