use std::path::PathBuf;

use clap::{CommandFactory, Parser};
use focal::{
    cli::{generate_completions, CaptureArea, Cli},
    create_parent_dirs, iso8601_filename, Screencast, Screenshot,
};

/// check if all required programs are installed
fn check_programs(args: &Cli) {
    let mut progs = std::collections::HashSet::from(["wl-copy", "xdg-open"]);

    if args.video_args.video {
        progs.insert("wf-recorder");
    } else {
        progs.insert("grim");
    }

    if args.rofi_args.rofi {
        progs.insert("rofi");
        progs.insert("slurp");
    }

    if matches!(args.area, Some(CaptureArea::Selection)) {
        progs.insert("slurp");
    }

    if args.image_args.ocr.is_some() {
        progs.insert("tesseract");
    }

    let not_found: Vec<_> = progs
        .into_iter()
        .filter(|prog| which::which(prog).is_err())
        .collect();

    if !not_found.is_empty() {
        eprintln!(
            "The following programs are required but not installed: {}",
            not_found.join(", ")
        );
        std::process::exit(1);
    }
}

fn main() {
    let args = Cli::parse();

    // print shell completions
    if let Some(shell) = args.generate {
        return generate_completions(&shell);
    }

    if !cfg!(feature = "ocr") && args.image_args.ocr.is_some() {
        Cli::command()
            .error(
                clap::error::ErrorKind::UnknownArgument,
                "OCR support was not built in this version of focal.",
            )
            .exit()
    }

    // check if all required programs are installed
    check_programs(&args);

    // stop any currently recording videos
    if args.video_args.video && Screencast::stop(!args.no_notify) {
        println!("Stopping previous recording...");
        return;
    }

    if args.video_args.video {
        let fname = format!("{}.mp4", iso8601_filename());

        let output = if args.no_save {
            PathBuf::from(format!("/tmp/{fname}"))
        } else {
            create_parent_dirs(args.filename.unwrap_or_else(|| {
                dirs::video_dir()
                    .expect("could not get $XDG_VIDEOS_DIR")
                    .join(format!("Screencasts/{fname}"))
            }))
        };

        let mut screencast = Screencast {
            output,
            icons: !args.rofi_args.no_icons,
            delay: args.delay,
            audio: args.video_args.audio,
            slurp: args.slurp,
        };

        if args.rofi_args.rofi {
            screencast.rofi(&args.rofi_args.theme);
        } else if let Some(area) = args.area {
            match area {
                CaptureArea::Monitor => screencast.monitor(),
                CaptureArea::Selection => screencast.selection(),
                CaptureArea::All => {
                    unimplemented!("Capturing of all outputs has not been implemented for video")
                }
            }
        }
    } else {
        let fname = format!("{}.png", iso8601_filename());

        let output = if args.no_save {
            PathBuf::from(format!("/tmp/{fname}"))
        } else {
            create_parent_dirs(args.filename.unwrap_or_else(|| {
                dirs::picture_dir()
                    .expect("could not get $XDG_PICTURES_DIR")
                    .join(format!("Screenshots/{fname}"))
            }))
        };

        let mut screenshot = Screenshot {
            output,
            delay: args.delay,
            edit: args.image_args.edit,
            icons: !args.rofi_args.no_icons,
            notify: !args.no_notify,
            ocr: args.image_args.ocr,
            slurp: args.slurp,
        };

        if args.rofi_args.rofi {
            screenshot.rofi(&args.rofi_args.theme);
        } else if let Some(area) = args.area {
            match area {
                CaptureArea::Monitor => screenshot.monitor(),
                CaptureArea::Selection => screenshot.selection(),
                CaptureArea::All => screenshot.all(),
            }
        }
    }
}
