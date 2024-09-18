use crate::{image::ImageArgs, video::VideoArgs};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};

#[derive(Subcommand, Debug)]
pub enum FocalSubcommands {
    #[command(name = "image", about = "Captures a screenshot.")]
    Image(ImageArgs),

    #[command(name = "video", about = "Captures a video.")]
    Video(VideoArgs),

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

    #[arg(long, action, help = "Do not show notifications")]
    pub no_notify: bool,

    #[arg(long, action, help = "Do not save the file permanently")]
    pub no_save: bool,
}

#[derive(Parser, Debug)]
#[command(
    name = "focal",
    about = "focal is a rofi menu for capturing and copying screenshots or videos on hyprland / sway.",
    author,
    version = env!("CARGO_PKG_VERSION"),
    flatten_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: FocalSubcommands,
}

pub fn generate_completions(shell_completion: &ShellCompletion) {
    let mut cmd = Cli::command();

    match shell_completion {
        ShellCompletion::Bash => generate(Shell::Bash, &mut cmd, "focal", &mut std::io::stdout()),
        ShellCompletion::Zsh => generate(Shell::Zsh, &mut cmd, "focal", &mut std::io::stdout()),
        ShellCompletion::Fish => generate(Shell::Fish, &mut cmd, "focal", &mut std::io::stdout()),
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
