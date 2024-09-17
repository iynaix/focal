use clap::{ArgGroup, Args};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    check_programs,
    cli::{CaptureArea, CommonArgs},
    create_parent_dirs, iso8601_filename,
    monitor::FocalMonitors,
    rofi::RofiArgs,
    wf_recorder::WfRecorder,
    Monitors, Rofi, SlurpGeom,
};
use execute::{command, Execute};

#[allow(clippy::module_name_repetitions)]
#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("mode")
        .multiple(false)
        .args(["rofi", "area", "stop"]),
))]
pub struct VideoArgs {
    #[command(flatten)]
    pub common_args: CommonArgs,

    #[command(flatten)]
    pub rofi_args: RofiArgs,

    #[arg(long, action, help = "Capture video with audio")]
    pub audio: bool,

    #[arg(long, action, help = "Stops any previous video recordings")]
    pub stop: bool,

    #[arg(
        name = "FILE",
        help = "Files are created in XDG_VIDEOS_DIR/Screencasts if not specified"
    )]
    pub filename: Option<PathBuf>,
}

impl VideoArgs {
    pub fn required_programs(&self) -> Vec<&str> {
        let mut progs = vec!["wf-recorder", "pkill"];

        if self.rofi_args.rofi {
            progs.push("rofi");
            progs.push("slurp");
        }

        if matches!(self.common_args.area, Some(CaptureArea::Selection)) {
            progs.push("slurp");
        }

        progs
    }
}

#[derive(Serialize, Deserialize)]
pub struct LockFile {
    pub video: PathBuf,
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
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    pub fn remove() {
        if Self::exists() {
            std::fs::remove_file(Self::path()).expect("failed to delete focal.lock");
        }
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

        let mon = Monitors::focused();
        self.capture(&mon.name, &mon.rotation.ffmpeg_transpose());
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
        delay: args.common_args.delay,
        audio: args.audio,
        slurp: args.common_args.slurp,
    };

    if args.rofi_args.rofi {
        screencast.rofi(&args.rofi_args.theme);
    } else if let Some(area) = args.common_args.area {
        match area {
            CaptureArea::Monitor => screencast.monitor(),
            CaptureArea::Selection => screencast.selection(),
            CaptureArea::All => {
                unimplemented!("Capturing of all outputs has not been implemented for video")
            }
        }
    }
}
