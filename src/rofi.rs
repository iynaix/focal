use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use clap::Args;
use execute::Execute;

#[allow(clippy::module_name_repetitions)]
#[derive(Args, Debug)]
pub struct RofiArgs {
    #[arg(long, action, help = "Display rofi menu for selection options")]
    pub rofi: bool,

    #[arg(long, action, help = "Do not show icons for rofi menu")]
    pub no_icons: bool,

    #[arg(long, action, help = "Path to a rofi theme")]
    pub theme: Option<PathBuf>,
}

pub struct Rofi {
    choices: Vec<String>,
    command: Command,
    message: String,
    theme: PathBuf,
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
            theme: dirs::cache_dir()
                .expect("could not get $XDG_CACHE_HOME")
                .join("wallust/rofi-menu-noinput.rasi"),
        }
    }

    #[must_use]
    pub fn arg<S: AsRef<std::ffi::OsStr>>(mut self, arg: S) -> Self {
        self.command.arg(arg);
        self
    }

    #[must_use]
    pub fn theme(mut self, theme: PathBuf) -> Self {
        self.theme = theme;
        self
    }

    #[must_use]
    pub fn message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn run(self) -> (String, i32) {
        let mut cmd = self.command;

        if self.theme.exists() {
            cmd.arg("-theme").arg(self.theme);
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
            .execute_input_output(self.choices.join("|").as_bytes())
            .expect("failed to run rofi");

        let exit_code = output.status.code().expect("rofi has not exited");
        let selection = std::str::from_utf8(&output.stdout)
            .expect("failed to parse utf8 from rofi selection")
            .strip_suffix('\n')
            .unwrap_or_default()
            .to_string();

        (selection, exit_code)
    }
}
