use color_eyre::eyre::{OptionExt, Result};
use std::process::{Command, Stdio};

use execute::Execute;

use crate::{
    focal_monitor, focal_screenshot,
    screenshot::{FocalScreenshot, FocalScreenshotArgs},
};

pub struct NoctaliaDmenu {
    choices: Vec<String>,
    command: Command,
    prompt: String,
}

impl NoctaliaDmenu {
    pub fn new<S>(choices: &[S]) -> Self
    where
        S: AsRef<str>,
    {
        let mut cmd = Command::new("noctalia");
        cmd.arg("dmenu");

        Self {
            choices: choices.iter().map(|s| s.as_ref().to_string()).collect(),
            command: cmd,
            prompt: String::new(),
        }
    }

    #[must_use]
    pub fn arg<S: AsRef<std::ffi::OsStr>>(mut self, arg: S) -> Self {
        self.command.arg(arg);
        self
    }

    #[must_use]
    pub fn prompt(mut self, prompt: &str) -> Self {
        self.prompt = prompt.to_string();
        self
    }

    pub fn run(self) -> Result<(String, i32)> {
        let mut cmd = self.command;

        if !self.prompt.is_empty() {
            cmd.arg("-p").arg(&self.prompt);
        }

        let output = cmd
            .stdout(Stdio::piped())
            // use newline as separator
            .execute_input_output(self.choices.join("\n").as_bytes())?;

        let exit_code = output
            .status
            .code()
            .ok_or_eyre("noctalia dmenu has not exited")?;
        let selection = std::str::from_utf8(&output.stdout)?
            .strip_suffix('\n')
            .unwrap_or_default()
            .to_string();

        Ok((selection, exit_code))
    }
}

pub fn screenshot_menu(delay: Option<u64>, screenshot_args: &FocalScreenshotArgs) -> Result<()> {
    let mut opts = vec!["Selection", "Window", "Monitor"];

    // don't show "All" option if single monitor
    if focal_monitor().all()?.len() > 1 {
        opts.push("All");
    }

    let noctalia_dmenu = NoctaliaDmenu::new(&opts);
    let (sel, _) = noctalia_dmenu.prompt("Select region to screenshot").run()?;

    let sel = sel
        .split('\t')
        .collect::<Vec<&str>>()
        .pop()
        .unwrap_or_default();

    // show menu to select delay if needed
    let delay = match delay {
        Some(d) => d,
        None => delay_menu()?,
    };

    let screenshot_args = FocalScreenshotArgs {
        delay,
        ..screenshot_args.clone()
    };

    match sel {
        "Selection" => focal_screenshot().selection(&screenshot_args)?,
        "Window" => focal_screenshot().window(&screenshot_args)?,
        "Monitor" => focal_screenshot().monitor(&screenshot_args)?,
        "All" => focal_screenshot().all(&screenshot_args)?,

        "" => {
            eprintln!("No capture selection was made.");
            std::process::exit(1);
        }
        _ => unimplemented!("Invalid selection"),
    };

    Ok(())
}

/// prompts the user for delay using rofi if not provided as a cli flag
fn delay_menu() -> Result<u64> {
    let delay_options = ["0s", "3s", "5s", "10s"];

    let noctalia_dmenu = NoctaliaDmenu::new(&delay_options).prompt("Select a delay");

    // disable the animations on hyprland so the delay rofi closing animation will not be captured by hyprpicker
    let (sel, _) = noctalia_dmenu.run()?;

    if sel.is_empty() {
        eprintln!("No delay selection was made.");
        std::process::exit(1);
    }

    Ok(sel.replace('s', "").parse::<u64>()?)
}

pub struct NoctaliaScreenshot;

impl FocalScreenshot for NoctaliaScreenshot {
    fn monitor(&self, args: &FocalScreenshotArgs) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(args.delay));
        Command::new("noctalia")
            .arg("msg")
            .arg("screenshot-fullscreen")
            .stdout(Stdio::piped())
            .output()?;
        Ok(())
    }

    fn window(&self, args: &FocalScreenshotArgs) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(args.delay));
        Command::new("noctalia")
            .arg("msg")
            .arg("screenshot-window")
            .stdout(Stdio::piped())
            .output()?;
        Ok(())
    }

    fn selection(&self, args: &FocalScreenshotArgs) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(args.delay));
        Command::new("noctalia")
            .arg("msg")
            .arg("screenshot-region")
            .stdout(Stdio::piped())
            .output()?;
        return Ok(());
    }

    fn all(&self, args: &FocalScreenshotArgs) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(args.delay));
        Command::new("noctalia")
            .arg("msg")
            .arg("screenshot-fullscreen")
            .arg("all")
            .stdout(Stdio::piped())
            .output()?;

        return Ok(());
    }
}
