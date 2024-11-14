use super::GenerateArgs;
use clap::{Parser, Subcommand};

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
pub struct Cli {
    // subcommand for generating shell completions
    #[command(subcommand)]
    pub command: Option<FocalWaybarSubcommands>,

    #[arg(
        long,
        value_name = "MESSAGE",
        default_value = "REC",
        help = "Message to display in waybar module when recording"
    )]
    pub recording: String,

    #[arg(
        long,
        value_name = "MESSAGE",
        default_value = "",
        help = "Message to display in waybar module when not recording"
    )]
    pub stopped: String,
}
