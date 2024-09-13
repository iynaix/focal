use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{monitor::FocalMonitors, show_notification, Monitors, Rofi, SlurpGeom};
use execute::{command, Execute};

#[allow(clippy::module_name_repetitions)]
#[derive(Args, Debug)]
#[command(next_help_heading = "Video Options")]
pub struct VideoArgs {
    #[arg(
        long,
        action,
        id = "video",
        help = "Records video / stops previous recordings",
        long_help = "Records video instead of screenshots\nRunning a second time stops any previous recordings"
    )]
    pub video: bool,

    #[arg(long, action, help = "Capture video with audio", requires = "video")]
    pub audio: bool,
}

#[derive(Serialize, Deserialize)]
struct LockFile {
    video: PathBuf,
}

impl LockFile {
    fn path() -> PathBuf {
        dirs::runtime_dir()
            .expect("could not get $XDG_RUNTIME_DIR")
            .join("focal.lock")
    }

    fn write(&self) -> std::io::Result<()> {
        let content = serde_json::to_string(&self).expect("failed to serialize focal.lock");
        std::fs::write(Self::path(), content)
    }

    fn read() -> std::io::Result<Self> {
        let content = std::fs::read_to_string(Self::path())?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn remove() {
        std::fs::remove_file(Self::path()).expect("failed to delete focal.lock");
    }
}

#[derive(Default)]
struct WfRecorder {
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
        let mut sys = sysinfo::System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

        let mut is_killed = false;
        sys.processes_by_exact_name("wf-recorder".as_ref())
            .filter(|p| p.parent().map_or(false, |parent| parent.as_u32() == 1))
            .for_each(|p| {
                is_killed = true;
                p.kill_with(sysinfo::Signal::Interrupt);
            });

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

        if !self.filter.is_empty() {
            wfrecorder.arg("--filter").arg(&self.filter);
        }

        if self.audio {
            wfrecorder.arg("--audio");
        }

        if wfrecorder
            .arg("--output")
            .arg(&self.monitor)
            .arg("--overwrite")
            .arg("-f")
            .arg(&self.video)
            .spawn()
            .is_ok()
        {
            // duration provied, recording will stop by itself so no lock file is needed
            if let Some(duration) = self.duration {
                std::thread::sleep(std::time::Duration::from_secs(duration));

                Self::stop(self.notify);
            } else {
                let lock = LockFile { video: self.video };
                lock.write().expect("failed to write to focal.lock");
            }
        } else {
            panic!("failed to execute wf-recorder");
        }
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

pub struct Screencast {
    pub delay: Option<u64>,
    pub icons: bool,
    pub audio: bool,
    pub slurp: Option<String>,
    pub output: PathBuf,
}

impl Screencast {
    fn capture(&self, mon: &str, filter: &str) {
        // copy the video file to clipboard
        command!("wl-copy")
            .arg("--type")
            .arg("text/uri-list")
            .execute_input(&format!("file://{}", self.output.display()))
            .expect("failed to copy video to clipboard");

        // small delay before recording
        std::thread::sleep(std::time::Duration::from_millis(500));

        WfRecorder::new(mon, self.output.clone())
            .audio(self.audio)
            .filter(filter)
            .record();
    }

    pub fn selection(&self) {
        let (mon, filter) = SlurpGeom::prompt(&self.slurp).to_ffmpeg_geom();

        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));
        self.capture(&mon, &filter);
    }

    pub fn monitor(&self) {
        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));
        self.capture(&Monitors::focused().name, "");
    }

    pub fn stop(notify: bool) -> bool {
        LockFile::read().map_or_else(|_| false, |_| WfRecorder::stop(notify))
    }

    pub fn rofi(&mut self, theme: &Option<PathBuf>) {
        let mut opts = vec!["Selection", "Monitor"];

        if !self.icons {
            opts = opts
                .iter()
                .map(|s| s.split('\t').collect::<Vec<&str>>()[1])
                .collect();
        }

        let mut rofi = Rofi::new(&opts);

        if let Some(theme) = &theme {
            rofi = rofi.theme(theme.clone());
        }

        let (sel, exit_code) = rofi
            // record audio with Alt+a
            .arg("-kb-custom-1")
            .arg("Alt-a")
            .message("Audio can be recorded using Alt+a")
            .run();

        // custom keyboard code selected
        if !self.audio {
            self.audio = exit_code == 10;
        }

        let sel = sel
            .split('\t')
            .collect::<Vec<&str>>()
            .pop()
            .unwrap_or_default();

        match sel {
            "Monitor" => {
                self.delay = Some(Self::rofi_delay(theme));
                self.monitor();
            }
            "Selection" => {
                self.delay = Some(Self::rofi_delay(theme));
                self.selection();
            }
            "All" => unimplemented!("Capturing of all outputs has not been implemented for video"),
            "" => {
                eprintln!("No rofi selection was made.");
                std::process::exit(1);
            }
            _ => unimplemented!("Invalid rofi selection"),
        };
    }

    /// prompts the user for delay using rofi if not provided as a cli flag
    fn rofi_delay(theme: &Option<PathBuf>) -> u64 {
        let delay_options = ["0s", "3s", "5s", "10s"];

        let mut rofi = Rofi::new(&delay_options).message("Select a delay");
        if let Some(theme) = theme {
            rofi = rofi.theme(theme.clone());
        }

        let (sel, _) = rofi.run();

        if sel.is_empty() {
            eprintln!("No delay selection was made.");
            std::process::exit(1);
        }

        sel.replace('s', "")
            .parse::<u64>()
            .expect("Invalid delay specified")
    }
}
