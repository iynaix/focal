use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use color_eyre::eyre::{OptionExt, Result};
use execute::Execute;
use libwayshot::WayshotConnection;

use crate::{focal_monitor, is_niri, show_notification};

#[derive(Debug, Clone)]
pub struct FocalScreenshotArgs {
    pub delay: u64,
    pub edit: Option<String>,
    pub ocr: Option<String>,
    pub noctalia: bool,
    pub notify: bool,
    pub slurp: Option<String>,
    pub output: PathBuf,
}

pub trait FocalScreenshot {
    fn edit(&self, args: &FocalScreenshotArgs) -> Result<()> {
        if let Some(prog) = &args.edit {
            std::process::Command::new(prog)
                .arg(args.output.clone())
                .execute()?;
        }

        // copying not needed for niri, since it already does that
        if !is_niri() {
            let mut img = std::fs::File::open(&args.output)?;
            Command::new("wl-copy")
                .arg("--type")
                .arg("image/png")
                .execute_input_reader(&mut img)?;
        }

        if args.notify {
            show_notification(
                &format!("Screenshot captured to {}", args.output.display()),
                Some(&args.output),
            )?;
        }

        Ok(())
    }

    fn ocr(&self, args: &FocalScreenshotArgs) -> Result<()> {
        let mut cmd = Command::new("tesseract");
        cmd.arg(&args.output).arg("-");

        if let Some(lang) = &args.ocr
            && !lang.is_empty()
        {
            cmd.arg("-l").arg(lang);
        }

        let output = cmd.stdout(Stdio::piped()).execute_output()?;

        Command::new("wl-copy").execute_input(&output.stdout)?;

        if args.notify
            && let Ok(copied_text) = std::str::from_utf8(&output.stdout)
        {
            return show_notification(copied_text, None);
        }

        Ok(())
    }

    fn edit_or_ocr(&self, args: &FocalScreenshotArgs) -> Result<()> {
        if args.ocr.is_some() {
            return self.ocr(args);
        } else if args.edit.is_some() {
            return self.edit(args);
        }
        Ok(())
    }

    fn monitor(&self, args: &FocalScreenshotArgs) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(args.delay));

        let curr_mon = &focal_monitor().focused()?.name;

        let conn = WayshotConnection::new()?;

        let focused = conn
            .get_all_outputs()
            .iter()
            .find(|mon| mon.name == *curr_mon)
            .ok_or_eyre("Could not get focused monitor")?;

        let img = conn.screenshot_single_output(focused, false)?;
        img.save(&args.output)?;

        self.edit_or_ocr(args)
    }

    fn window(&self, args: &FocalScreenshotArgs) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(args.delay));

        let conn = WayshotConnection::new()?;
        let slurp_args = args.slurp.clone();

        let window_geoms = focal_monitor().window_geoms().unwrap_or_default();
        let img = conn.screenshot_freeze(
            move |_| {
                let geom = crate::SlurpGeom::prompt(&window_geoms, slurp_args.as_deref())
                    .map_err(|e| libwayshot::Error::FreezeCallbackError(e.to_string()))?;

                Ok(geom.to_logical_region())
            },
            false,
        )?;
        img.save(&args.output)?;

        self.edit_or_ocr(args)
    }

    fn selection(&self, args: &FocalScreenshotArgs) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(args.delay));

        let conn = WayshotConnection::new()?;
        let slurp_args = args.slurp.clone();

        let img = conn.screenshot_freeze(
            move |_| {
                let geom = crate::SlurpGeom::prompt(&[], slurp_args.as_deref())
                    .map_err(|e| libwayshot::Error::FreezeCallbackError(e.to_string()))?;

                Ok(geom.to_logical_region())
            },
            false,
        )?;
        img.save(&args.output)?;

        self.edit_or_ocr(args)
    }

    fn all(&self, args: &FocalScreenshotArgs) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(args.delay));

        let conn = WayshotConnection::new()?;
        let img = conn.screenshot_all(false)?;
        img.save(&args.output)?;

        self.edit_or_ocr(args)
    }
}
