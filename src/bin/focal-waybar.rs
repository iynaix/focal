use clap::{CommandFactory, Parser, Subcommand};
use focal::{
    cli::{generate_completions, GenerateArgs},
    video::LockFile,
};
use std::env;

#[derive(Subcommand, Debug)]
pub enum FocalWaybarSubcommands {
    #[command(name = "generate", about = "Generate shell completions", hide = true)]
    Generate(GenerateArgs),
}

#[derive(Parser, Debug)]
#[command(
    name = "focal-waybar",
    about = "Updates waybar module with focal's recording status.",
    author,
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    // subcommand for generating shell completions
    #[command(subcommand)]
    pub command: Option<FocalWaybarSubcommands>,

    #[arg(
        long,
        value_name = "MESSAGE",
        default_value = "REC",
        help = "Message to display in waybar module when recording"
    )]
    recording: String,

    #[arg(
        long,
        value_name = "MESSAGE",
        default_value = "",
        help = "Message to display in waybar module when not recording"
    )]
    stopped: String,
}

fn main() {
    let args = Cli::parse();

    if let Some(FocalWaybarSubcommands::Generate(args)) = args.command {
        generate_completions("focal-waybar", &mut Cli::command(), &args.shell);
        return;
    }

    let output = if LockFile::exists() {
        args.recording
    } else {
        args.stopped
    };
    println!("{output}");
}
