use crate::is_hyprland;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use color_eyre::eyre::{OptionExt, Result};
use execute::Execute;

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

    /// runs rofi without animations, re-enabling the animation afterwards if needed
    pub fn run_without_animation(self) -> Result<(String, i32)> {
        use hyprland::keyword::{Keyword, OptionValue};

        if is_hyprland()
            && let Ok(Keyword {
                value: OptionValue::Int(1),
                ..
            }) = Keyword::get("animations:enabled")
        {
            Keyword::set("animations:enabled", 0)?;
            let ret = self.run();
            Keyword::set("animations:enabled", 1)?;
            return ret;
        } else {
            self.run()
        }
    }
}
