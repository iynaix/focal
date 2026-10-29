use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use color_eyre::eyre::{OptionExt, Result};
use execute::Execute;

use crate::{focal_monitor, focal_screenshot, is_mango, is_niri, screenshot::FocalScreenshotArgs};

pub struct Rofi {
    choices: Vec<String>,
    command: Command,
    message: String,
    theme: Option<PathBuf>,
}

impl Rofi {
    pub fn new<S>(choices: &[S]) -> Self
    where
        S: AsRef<str>,
    {
        let mut cmd = Command::new("rofi");

        cmd.arg("-dmenu")
            // hide the search input
            .arg("-theme-str")
            .arg("mainbox { children: [listview, message]; }")
            // use | as separator
            .arg("-sep")
            .arg("|")
            .arg("-disable-history")
            .arg("true")
            .arg("-cycle")
            .arg("true");

        Self {
            choices: choices.iter().map(|s| s.as_ref().to_string()).collect(),
            command: cmd,
            message: String::new(),
            theme: None,
        }
    }

    #[must_use]
    pub fn arg<S: AsRef<std::ffi::OsStr>>(mut self, arg: S) -> Self {
        self.command.arg(arg);
        self
    }

    #[must_use]
    pub fn theme(mut self, theme: PathBuf) -> Self {
        self.theme = Some(theme);
        self
    }

    #[must_use]
    pub fn message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn run(self) -> Result<(String, i32)> {
        let mut cmd = self.command;

        if let Some(theme) = self.theme
            && theme.exists()
        {
            cmd.arg("-theme").arg(theme);
        }

        if !self.message.is_empty() {
            cmd.arg("-mesg").arg(&self.message);
        }

        // hide the search input, show message if necessary
        cmd.arg("-theme-str").arg(format!(
            "mainbox {{ children: {}; }}",
            if self.message.is_empty() {
                "[ listview ]"
            } else {
                "[ listview, message ]"
            }
        ));

        let output = cmd
            .stdout(Stdio::piped())
            // use | as separator
            .execute_input_output(self.choices.join("|").as_bytes())?;

        let exit_code = output.status.code().ok_or_eyre("rofi has not exited")?;
        let selection = std::str::from_utf8(&output.stdout)?
            .strip_suffix('\n')
            .unwrap_or_default()
            .to_string();

        Ok((selection, exit_code))
    }
}

/// prompts the user for delay using rofi if not provided as a cli flag
fn delay_menu(theme: Option<&PathBuf>) -> Result<u64> {
    let delay_options = ["0s", "3s", "5s", "10s"];

    let mut rofi = Rofi::new(&delay_options).message("Select a delay");
    if let Some(theme) = theme {
        rofi = rofi.theme(theme.clone());
    }

    // disable the animations on hyprland so the delay rofi closing animation will not be captured by hyprpicker
    let (sel, _) = rofi.run()?;

    if sel.is_empty() {
        eprintln!("No delay selection was made.");
        std::process::exit(1);
    }

    Ok(sel.replace('s', "").parse::<u64>()?)
}

pub fn screenshot_menu(
    delay: Option<u64>,
    screenshot_args: &FocalScreenshotArgs,
    theme: Option<&PathBuf>,
    show_icons: bool,
) -> Result<()> {
    let mut opts = vec!["󰒉\tSelection", "󰍹\tWindow", "󰍹\tMonitor"];

    if !(is_niri() || is_mango()) {
        opts.push("󰍺\tAll");
    }

    // don't show "All" option if single monitor
    if focal_monitor().all()?.len() == 1 {
        opts.pop();
    }

    if !show_icons {
        opts = opts.iter().filter_map(|s| s.split('\t').nth(1)).collect();
    }

    let mut rofi = Rofi::new(&opts);

    if let Some(theme) = theme {
        rofi = rofi.theme(theme.clone());
    }

    let mut edit_final = screenshot_args.edit.clone();

    // only show edit message if an editor is provided
    let sel = if screenshot_args.edit.is_some() {
        let (sel, exit_code) = rofi
            .arg("-kb-custom-1")
            .arg("Alt-e")
            .message("Screenshots can be edited with Alt+e")
            .run()?;

        // no alt keycode selected, do not edit
        if exit_code != 10 {
            edit_final = None;
        }

        sel
    } else {
        rofi.run()?.0
    };

    let sel = sel
        .split('\t')
        .collect::<Vec<&str>>()
        .pop()
        .unwrap_or_default();

    // show menu to select delay if needed
    let delay = match delay {
        Some(d) => d,
        None => delay_menu(theme)?,
    };

    let screenshot_args = FocalScreenshotArgs {
        edit: edit_final,
        delay,
        ..screenshot_args.clone()
    };

    match sel {
        "Selection" => focal_screenshot().selection(&screenshot_args),
        "Window" => focal_screenshot().window(&screenshot_args),
        "Monitor" => focal_screenshot().monitor(&screenshot_args),
        "All" => focal_screenshot().all(&screenshot_args),
        "" => {
            eprintln!("No capture selection was made.");
            std::process::exit(1);
        }
        _ => {
            eprintln!("Invalid capture selection.");
            std::process::exit(1);
        }
    }
}
