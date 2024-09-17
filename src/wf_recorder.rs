use execute::{command, Execute};
use std::{path::PathBuf, process::Stdio};

use crate::{show_notification, video::LockFile};

#[derive(Default)]
pub struct WfRecorder {
    monitor: String,
    audio: bool,
    notify: bool,
    video: PathBuf,
    filter: String,
    duration: Option<u64>,
}

impl WfRecorder {
    pub fn new(monitor: &str, video: PathBuf) -> Self {
        Self {
            monitor: monitor.to_string(),
            video,
            ..Default::default()
        }
    }

    pub const fn audio(mut self, audio: bool) -> Self {
        self.audio = audio;
        self
    }

    pub fn filter(mut self, filter: &str) -> Self {
        self.filter = filter.to_string();
        self
    }

    #[allow(dead_code)]
    pub const fn duration(mut self, seconds: u64) -> Self {
        self.duration = Some(seconds);
        self
    }

    pub fn stop(notify: bool) -> bool {
        // kill all wf-recorder processes
        let wf_process = std::process::Command::new("pkill")
            .arg("--echo")
            .arg("-SIGINT")
            .arg("wf-recorder")
            .output()
            .expect("failed to pkill wf-recorder")
            .stdout;

        let is_killed = String::from_utf8(wf_process)
            .expect("failed to parse pkill output")
            .lines()
            .count()
            > 0;

        if let Ok(LockFile { video }) = LockFile::read() {
            LockFile::remove();

            // show notification with the video thumbnail
            if notify {
                Self::notify(&video);
            }

            return true;
        }

        is_killed
    }

    pub fn record(self) {
        let mut wfrecorder = command!("wf-recorder");

        ctrlc::set_handler(move || {
            Self::stop(self.notify);
        })
        .expect("unable to set ctrl-c handler");

        if !self.filter.is_empty() {
            wfrecorder.arg("--filter").arg(&self.filter);
        }

        if self.audio {
            wfrecorder.arg("--audio");
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
            .expect("failed to spawn wf-recorder");

        // duration provied, recording will stop by itself so no lock file is needed
        if let Some(duration) = self.duration {
            std::thread::sleep(std::time::Duration::from_secs(duration));

            Self::stop(self.notify);
            return;
        }

        // write the lock file
        let lock = LockFile { video: self.video };
        lock.write().expect("failed to write to focal.lock");
    }

    fn notify(video: &PathBuf) {
        let thumb_path = PathBuf::from("/tmp/focal-thumbnail.jpg");

        if thumb_path.exists() {
            std::fs::remove_file(&thumb_path).expect("failed to remove notification thumbnail");
        }

        command!("ffmpeg")
            .arg("-i")
            .arg(video)
            // from 3s in the video
            .arg("-ss")
            .arg("00:00:03.000")
            .arg("-vframes")
            .arg("1")
            .arg("-s")
            .arg("128x72")
            .arg(&thumb_path)
            .execute()
            .expect("failed to create notification thumbnail");

        // show notifcation with the video thumbnail
        show_notification(
            &format!("Video captured to {}", video.display()),
            &thumb_path,
        );
    }
}
