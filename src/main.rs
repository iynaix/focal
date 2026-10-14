use clap::{CommandFactory, Parser};
use color_eyre::eyre::Result;
use focal::cli::focal::{Cli, FocalSubcommand, generate_completions};

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        FocalSubcommand::Generate(args) => {
            generate_completions("focal", &mut Cli::command(), &args.shell);
            return Ok(());
        }
        FocalSubcommand::Image(image_args) => focal::image::main(image_args),
        #[cfg(feature = "video")]
        FocalSubcommand::Video(video_args) => focal::video::main(video_args),
    }
}
