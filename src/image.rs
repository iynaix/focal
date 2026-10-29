use color_eyre::eyre::{OptionExt, Result};
use std::path::PathBuf;

use crate::{
    check_programs,
    cli::{
        focal::Cli,
        image::{CaptureArea, ImageArgs},
    },
    create_parent_dirs, focal_screenshot, iso8601_filename,
    screenshot::FocalScreenshotArgs,
};
use clap::CommandFactory;

pub fn main(args: ImageArgs) -> Result<()> {
    if !cfg!(feature = "ocr") && args.ocr.is_some() {
        Cli::command()
            .error(
                clap::error::ErrorKind::UnknownArgument,
                "OCR support was not built in this version of focal.",
            )
            .exit()
    }

    let has_noctalia = which::which("noctalia").is_ok();

    // check if all required programs are installed
    check_programs(&args.required_programs(has_noctalia));

    let fname = format!("{}.png", iso8601_filename());

    let output = if args.common_args.no_save {
        PathBuf::from(format!("/tmp/{fname}"))
    } else {
        let path = match args.filename {
            Some(filename) => filename,
            None => {
                let picture_dir =
                    dirs::picture_dir().ok_or_eyre("could not get $XDG_PICTURES_DIR")?;

                picture_dir.join(format!("Screenshots/{fname}"))
            }
        };
        create_parent_dirs(path)?
    };

    let screenshot_args = FocalScreenshotArgs {
        delay: args.common_args.delay.unwrap_or_default(),
        edit: args.edit,
        ocr: args.ocr,
        notify: !args.common_args.no_notify,
        slurp: args.common_args.slurp,
        output,
    };

    if args.menu_args.menu {
        // use noctalia if available
        if has_noctalia {
            return crate::noctalia::screenshot_menu(args.common_args.delay, &screenshot_args);
        } else {
            return crate::rofi::screenshot_menu(
                args.common_args.delay,
                &screenshot_args,
                args.menu_args.theme.as_ref(),
                !args.menu_args.no_icons,
            );
        }
    } else if let Some(area) = args.area_args.parse() {
        return match area {
            CaptureArea::Monitor => focal_screenshot().monitor(&screenshot_args),
            CaptureArea::Window => focal_screenshot().window(&screenshot_args),
            CaptureArea::Selection => focal_screenshot().selection(&screenshot_args),
            CaptureArea::All => focal_screenshot().all(&screenshot_args),
        };
    }

    Ok(())
}
