use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

#[derive(Default)]
pub struct WfRecorder {
    monitor: String,
    audio: Option<String>,
    video: PathBuf,
    filter: String,
}

impl WfRecorder {
    pub fn new(monitor: &str, video: PathBuf) -> Self {
        Self {
            monitor: monitor.to_string(),
            video,
            ..Default::default()
        }
    }

    pub fn audio(mut self, audio: Option<&str>) -> Self {
        self.audio = audio.map(std::string::ToString::to_string);
        self
    }

    pub fn filter(mut self, filter: &str) -> Self {
        self.filter = filter.to_string();
        self
    }

    pub fn record(self) {
        let mut wfrecorder = Command::new("wf-recorder");

        if !self.filter.is_empty() {
            wfrecorder.arg("--filter").arg(&self.filter);
        }

        if let Some(device) = &self.audio {
            wfrecorder.arg("--audio");

            if !device.is_empty() {
                wfrecorder.arg("--device").arg(device);
            }
        }

        wfrecorder
            .arg("--output")
            .arg(&self.monitor)
            .arg("--overwrite")
            .arg("-f")
            .arg(&self.video)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to spawn wf-recorder")
            .wait()
            .expect("failed to wait for wf-recorder");
    }
}
