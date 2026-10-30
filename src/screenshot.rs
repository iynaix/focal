use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use color_eyre::eyre::Result;
use execute::Execute;

use crate::{focal_monitor, grim::Grim, is_niri, show_notification};

#[derive(Debug, Clone)]
pub struct FocalScreenshotArgs {
    pub delay: u64,
    pub edit: Option<String>,
    pub ocr: Option<String>,
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

    fn capture(&self, monitor: &str, geometry: &str, args: &FocalScreenshotArgs) -> Result<()> {
        // small delay before capture
        std::thread::sleep(std::time::Duration::from_millis(500));

        let grim = Grim::new(args.output.clone())
            .geometry(geometry)
            .monitor(monitor);

        grim.capture()?;

        // wait for up to 5s for the file to appear
        let mut attempts = 0;
        while !args.output.exists() && attempts < 50 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            attempts += 1;
        }
        if !args.output.exists() {
            eprintln!("No image was captured by grim!");
            std::process::exit(1);
        }

        Ok(())
    }

    fn monitor(&self, args: &FocalScreenshotArgs) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(args.delay));
        self.capture(&focal_monitor().focused()?.name, "", args)?;
        self.edit_or_ocr(args)
    }

    fn window(&self, args: &FocalScreenshotArgs) -> Result<()> {
        self.selection(args)
    }

    fn selection(&self, args: &FocalScreenshotArgs) -> Result<()> {
        // freeze screen before delay to capture selection
        let picker_process = if args.delay > 0 {
            let child = Command::new("hyprpicker").arg("-rz").spawn()?;
            std::thread::sleep(std::time::Duration::from_millis(200));
            Some(child)
        } else {
            None
        };

        let geom = crate::SlurpGeom::prompt(args.slurp.as_deref())?;
        // kill hyprpicker so it doesn't ask for a color
        picker_process.map(|mut p| p.kill().ok());

        std::thread::sleep(std::time::Duration::from_secs(args.delay));
        self.capture("", &geom.to_string(), args)?;
        self.edit_or_ocr(args)
    }

    fn all(&self, args: &FocalScreenshotArgs) -> Result<()> {
        let (w, h) = focal_monitor().total_dimensions()?;

        std::thread::sleep(std::time::Duration::from_secs(args.delay));
        self.capture("", &format!("0,0 {w}x{h}"), args)?;
        self.edit_or_ocr(args)
    }
}
