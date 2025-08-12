use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Command, vec};

use crate::{
    Monitors, Rofi, SlurpGeom, check_programs,
    cli::video::{CaptureArea, VideoArgs},
    create_parent_dirs, iso8601_filename,
    monitor::FocalMonitors,
    show_notification,
    wf_recorder::WfRecorder,
};
use execute::Execute;

#[derive(Serialize, Deserialize)]
pub struct LockFile {
    pub video: PathBuf,
    pub rounding: Option<i64>,
}

impl LockFile {
    fn path() -> PathBuf {
        dirs::runtime_dir()
            .expect("could not get $XDG_RUNTIME_DIR")
            .join("focal.lock")
    }

    pub fn exists() -> bool {
        Self::path().exists()
    }

    pub fn write(&self) -> std::io::Result<()> {
        let content = serde_json::to_string(&self).expect("failed to serialize focal.lock");
        std::fs::write(Self::path(), content)
    }

    pub fn read() -> std::io::Result<Self> {
        let content = std::fs::read_to_string(Self::path())?;
        serde_json::from_str(&content).map_err(std::io::Error::other)
    }

    pub fn remove() {
        if Self::exists() {
            std::fs::remove_file(Self::path()).expect("failed to delete focal.lock");
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
pub struct Screencast {
    pub delay: Option<u64>,
    pub icons: bool,
    pub audio: Option<String>,
    pub no_rounded_windows: bool,
    pub notify: bool,
    pub duration: Option<u64>,
    pub slurp: Option<String>,
    pub output: PathBuf,
}

impl Screencast {
    fn capture(&self, mon: &str, filter: &str, rounding: Option<i64>) {
        ctrlc::set_handler(move || {
            Self::stop(false);
        })
        .expect("unable to set ctrl-c handler");

        // copy the video file to clipboard
        Command::new("wl-copy")
            .arg("--type")
            .arg("text/uri-list")
            .execute_input(&format!("file://{}", self.output.display()))
            .expect("failed to copy video to clipboard");

        // small delay before recording
        std::thread::sleep(std::time::Duration::from_millis(500));

        let lock = LockFile {
            video: self.output.clone(),
            rounding,
        };

        WfRecorder::new(mon, self.output.clone())
            .audio(self.audio.as_deref())
            .filter(filter)
            .record();

        // write the lock file
        lock.write().expect("failed to write to focal.lock");

        // duration provied, recording will stop by itself so no lock file is needed
        if let Some(duration) = self.duration {
            std::thread::sleep(std::time::Duration::from_secs(duration));

            Self::stop(false);
        }
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

        if let Ok(LockFile { video, rounding }) = LockFile::read() {
            LockFile::remove();

            if cfg!(feature = "hyprland") {
                if let Some(rounding) = rounding {
                    hyprland::keyword::Keyword::set("decoration:rounding", rounding)
                        .expect("unable to restore rounding");
                }
            }

            // show notification with the video thumbnail
            if notify {
                Self::notify(&video);
            }

            return true;
        }

        is_killed
    }

    fn notify(video: &PathBuf) {
        let thumb_path = PathBuf::from("/tmp/focal-thumbnail.jpg");

        if thumb_path.exists() {
            std::fs::remove_file(&thumb_path).expect("failed to remove notification thumbnail");
        }

        Command::new("ffmpeg")
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
            Some(&thumb_path),
        );
    }

    pub fn selection(&self) {
        let (geom, is_window) = SlurpGeom::prompt(self.slurp.as_deref());
        let (mon, filter) = geom.to_ffmpeg_geom();

        let do_capture = |rounding: Option<i64>| {
            std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));
            self.capture(&mon, &filter, rounding);
        };

        if cfg!(feature = "hyprland") && is_window && self.no_rounded_windows {
            use hyprland::keyword::{Keyword, OptionValue};

            if let Ok(Keyword {
                value: OptionValue::Int(rounding),
                ..
            }) = Keyword::get("decoration:rounding")
            {
                Keyword::set("decoration:rounding", 0).expect("unable to disable rounding");

                do_capture(Some(rounding));
            }
        }

        do_capture(None);
    }

    pub fn monitor(&self) {
        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));

        let mon = Monitors::focused();
        let transpose = mon.rotation.ffmpeg_transpose();
        self.capture(&mon.name, &transpose, None);
    }

    pub fn rofi(&mut self, theme: Option<&PathBuf>) {
        let mut opts = vec!["󰒉\tSelection", "󰍹\tMonitor", "󰍺\tAll"];

        // don't show "All" option if single monitor
        if Monitors::all().len() == 1 {
            opts.pop();
        }

        if !self.icons {
            opts = opts
                .iter()
                .map(|s| s.split('\t').collect::<Vec<&str>>()[1])
                .collect();
        }

        let mut rofi = Rofi::new(&opts);

        if let Some(theme) = theme {
            rofi = rofi.theme(theme.clone());
        }

        let (sel, exit_code) = rofi
            // record audio with Alt+a
            .arg("-kb-custom-1")
            .arg("Alt-a")
            .message("Audio can be recorded using Alt+a")
            .run();

        // custom keyboard code selected
        if self.audio.is_none() {
            self.audio = (exit_code == 10).then_some(String::new());
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
            "" => {
                eprintln!("No rofi selection was made.");
                std::process::exit(1);
            }
            _ => unimplemented!("Invalid rofi selection"),
        }
    }

    /// prompts the user for delay using rofi if not provided as a cli flag
    fn rofi_delay(theme: Option<&PathBuf>) -> u64 {
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

pub fn main(args: VideoArgs) {
    // stop any currently recording videos
    if Screencast::stop(!args.common_args.no_notify) {
        println!("Stopping previous recording...");
        return;
    }

    // nothing left to do
    if args.stop {
        return;
    }

    // check if all required programs are installed
    check_programs(&args.required_programs());

    let fname = format!("{}.mp4", iso8601_filename());

    let output = if args.common_args.no_save {
        PathBuf::from(format!("/tmp/{fname}"))
    } else {
        create_parent_dirs(args.filename.unwrap_or_else(|| {
            dirs::video_dir()
                .expect("could not get $XDG_VIDEOS_DIR")
                .join(format!("Screencasts/{fname}"))
        }))
    };

    let mut screencast = Screencast {
        output,
        icons: !args.rofi_args.no_icons,
        notify: !args.common_args.no_notify,
        no_rounded_windows: args.common_args.no_rounded_windows,
        delay: args.common_args.delay,
        duration: args.duration,
        audio: args.audio,
        slurp: args.common_args.slurp,
    };

    if args.rofi_args.rofi {
        screencast.rofi(args.rofi_args.theme.as_ref());
    } else if let Some(area) = args.area_args.parse() {
        match area {
            CaptureArea::Monitor => screencast.monitor(),
            CaptureArea::Selection => screencast.selection(),
        }
    }
}
