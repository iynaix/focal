use clap::{CommandFactory, Parser};
use focal::cli::{generate_completions, Cli, FocalSubcommand};

fn main() {
    let args = Cli::parse();

    match args.command {
        FocalSubcommand::Generate(args) => {
            generate_completions("focal", &mut Cli::command(), &args.shell);
        }
        FocalSubcommand::Image(image_args) => focal::image::main(image_args),
        #[cfg(feature = "video")]
        FocalSubcommand::Video(video_args) => focal::video::main(video_args),
    }
}
