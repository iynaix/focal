use clap::{CommandFactory, Parser};
use focal::{
    cli::{
        generate_completions,
        waybar::{Cli, FocalWaybarSubcommands},
    },
    video::LockFile,
};

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
