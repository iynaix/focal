use clap::Parser;
use focal::cli::{generate_completions, Cli, FocalSubcommands};

fn main() {
    let args = Cli::parse();

    match args.command {
        FocalSubcommands::Generate(args) => generate_completions(&args.shell),
        FocalSubcommands::Image(image_args) => focal::image::main(image_args),
        FocalSubcommands::Video(video_args) => focal::video::main(video_args),
    }
}
