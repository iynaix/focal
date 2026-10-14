use std::process::{Command, Stdio};

use execute::Execute;

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

    pub fn run(self) -> (String, i32) {
        let mut cmd = self.command;

        if !self.prompt.is_empty() {
            cmd.arg("-p").arg(&self.prompt);
        }

        let output = cmd
            .stdout(Stdio::piped())
            // use newline as separator
            .execute_input_output(self.choices.join("\n").as_bytes())
            .expect("failed to run noctalia dmenu");

        let exit_code = output.status.code().expect("noctalia dmenu has not exited");
        let selection = std::str::from_utf8(&output.stdout)
            .expect("failed to parse utf8 from noctalia dmenu selection")
            .strip_suffix('\n')
            .unwrap_or_default()
            .to_string();

        (selection, exit_code)
    }
}
