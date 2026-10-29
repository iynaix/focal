use color_eyre::eyre::{OptionExt, Result};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Command, vec};

use crate::{
    NoctaliaDmenu, Rofi, SlurpGeom, check_programs,
    cli::video::{CaptureArea, VideoArgs},
    create_parent_dirs, focal_monitor, iso8601_filename, show_notification,
    wf_recorder::WfRecorder,
};
use execute::Execute;

#[derive(Serialize, Deserialize)]
pub struct LockFile {
    pub video: PathBuf,
}

impl LockFile {
    fn path() -> Option<PathBuf> {
        dirs::runtime_dir().map(|d| d.join("focal.lock"))
    }

    pub fn write(&self) -> Result<()> {
        let path = Self::path().ok_or_eyre("could not get lockfile path")?;

        let content = serde_json::to_string(&self)?;
        Ok(std::fs::write(path, content)?)
    }

    pub fn read() -> Result<Self> {
        let path = Self::path().ok_or_eyre("could not get lockfile path")?;

        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content).map_err(std::io::Error::other)?)
    }

    pub fn remove() -> Result<()> {
        let path = Self::path().ok_or_eyre("could not get lockfile path")?;

        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

#[allow(clippy::struct_excessive_bools)]
pub struct Screencast {
    pub delay: Option<u64>,
    pub icons: bool,
    pub audio: Option<String>,
    pub notify: bool,
    pub duration: Option<u64>,
    pub slurp: Option<String>,
    pub output: PathBuf,
}

impl Screencast {
    fn capture(&self, mon: &str, filter: &str) -> Result<()> {
        ctrlc::set_handler(move || {
            if let Err(e) = Self::stop(false) {
                eprintln!("failed to stop: {e}");
            }
        })?;

        // copy the video file to clipboard
        Command::new("wl-copy")
            .arg("--type")
            .arg("text/uri-list")
            .execute_input(&format!("file://{}", self.output.display()))?;

        // small delay before recording
        std::thread::sleep(std::time::Duration::from_millis(500));

        let lock = LockFile {
            video: self.output.clone(),
        };

        WfRecorder::new(mon, self.output.clone())
            .audio(self.audio.as_deref())
            .filter(filter)
            .record()?;

        // write the lock file
        lock.write()?;

        // duration provied, recording will stop by itself so no lock file is needed
        if let Some(duration) = self.duration {
            std::thread::sleep(std::time::Duration::from_secs(duration));

            Self::stop(false)?;
        }

        Ok(())
    }

    pub fn stop(notify: bool) -> Result<bool> {
        // kill all wf-recorder processes
        let wf_process = std::process::Command::new("pkill")
            .arg("--echo")
            .arg("-SIGINT")
            .arg("wf-recorder")
            .output()?
            .stdout;

        let is_killed = String::from_utf8(wf_process)?.lines().count() > 0;

        if let Ok(LockFile { video }) = LockFile::read() {
            LockFile::remove()?;

            // show notification with the video thumbnail
            if notify {
                Self::notify(&video)?;
            }

            return Ok(true);
        }

        Ok(is_killed)
    }

    fn notify(video: &PathBuf) -> Result<()> {
        let thumb_path = PathBuf::from("/tmp/focal-thumbnail.jpg");

        if thumb_path.exists() {
            std::fs::remove_file(&thumb_path)?;
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
            .execute()?;

        // show notifcation with the video thumbnail
        show_notification(
            &format!("Video captured to {}", video.display()),
            Some(&thumb_path),
        )
    }

    pub fn selection(&self) -> Result<()> {
        let geom = SlurpGeom::prompt(self.slurp.as_deref())?;
        let (mon, filter) = geom.to_ffmpeg_geom()?;

        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));
        self.capture(&mon, &filter)
    }

    pub fn monitor(&self) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));

        let mon = focal_monitor().focused()?;
        let transpose = mon.rotation.ffmpeg_transpose();
        self.capture(&mon.name, &transpose)
    }

    pub fn rofi(&mut self, theme: Option<&PathBuf>) -> Result<()> {
        let mut opts = vec!["󰒉\tSelection", "󰍹\tMonitor"];

        // don't show "All" option if single monitor
        if focal_monitor().all()?.len() == 1 {
            opts.pop();
        }

        if !self.icons {
            opts = opts.iter().filter_map(|s| s.split('\t').nth(1)).collect();
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
            .run()?;

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
                self.delay = Some(Self::rofi_delay(theme)?);
                self.monitor()
            }
            "Selection" => {
                self.delay = Some(Self::rofi_delay(theme)?);
                self.selection()
            }
            "" => {
                eprintln!("No selection was made.");
                std::process::exit(1);
            }
            _ => unimplemented!("Invalid rofi selection"),
        }
    }

    /// prompts the user for delay using gui if not provided as a cli flag
    fn rofi_delay(theme: Option<&PathBuf>) -> Result<u64> {
        let delay_options = ["0s", "3s", "5s", "10s"];

        let mut rofi = Rofi::new(&delay_options).message("Select a delay");
        if let Some(theme) = theme {
            rofi = rofi.theme(theme.clone());
        }

        let (sel, _) = rofi.run()?;

        if sel.is_empty() {
            eprintln!("No delay selection was made.");
            std::process::exit(1);
        }

        Ok(sel.replace('s', "").parse::<u64>()?)
    }

    pub fn noctalia_dmenu(&mut self) -> Result<()> {
        let opts = vec!["󰒉\tSelection", "󰍹\tMonitor"];

        let noctalia_dmenu = NoctaliaDmenu::new(&opts);
        let (sel, _) = noctalia_dmenu
            // record audio with Alt+a
            // .arg("-kb-custom-1")
            // .arg("Alt-a")
            // .message("Audio can be recorded using Alt+a")
            .prompt("Select region to record")
            .run()?;

        // custom keyboard code selected
        // if self.audio.is_none() {
        //     self.audio = (exit_code == 10).then_some(String::new());
        // }

        let sel = sel
            .split('\t')
            .collect::<Vec<&str>>()
            .pop()
            .unwrap_or_default();

        return match sel {
            "Monitor" => {
                self.delay = Some(Self::noctalia_dmenu_delay()?);
                self.monitor()
            }
            "Selection" => {
                self.delay = Some(Self::noctalia_dmenu_delay()?);
                self.selection()
            }
            "" => {
                eprintln!("No selection was made.");
                std::process::exit(1);
            }
            _ => unimplemented!("Invalid selection"),
        };
    }

    /// prompts the user for delay using gui if not provided as a cli flag
    fn noctalia_dmenu_delay() -> Result<u64> {
        let delay_options = ["0s", "3s", "5s", "10s"];

        let rofi = Rofi::new(&delay_options).message("Select a delay");
        let (sel, _) = rofi.run()?;

        if sel.is_empty() {
            eprintln!("No delay selection was made.");
            std::process::exit(1);
        }

        Ok(sel.replace('s', "").parse::<u64>()?)
    }
}

pub fn main(args: VideoArgs) -> Result<()> {
    // stop any currently recording videos
    if Screencast::stop(!args.common_args.no_notify)? {
        println!("Stopping previous recording...");
        return Ok(());
    }

    // nothing left to do
    if args.stop {
        return Ok(());
    }

    // check if all required programs are installed
    check_programs(&args.required_programs());

    let fname = format!("{}.mp4", iso8601_filename());

    let output = if args.common_args.no_save {
        PathBuf::from(format!("/tmp/{fname}"))
    } else {
        let path = match args.filename {
            Some(filename) => filename,
            None => {
                let picture_dir =
                    dirs::picture_dir().ok_or_eyre("could not get $XDG_VIDEOS_DIR")?;

                picture_dir.join(format!("Screenshots/{fname}"))
            }
        };
        create_parent_dirs(path)?
    };

    let mut screencast = Screencast {
        output,
        icons: !args.menu_args.no_icons,
        notify: !args.common_args.no_notify,
        delay: args.common_args.delay,
        duration: args.duration,
        audio: args.audio,
        slurp: args.common_args.slurp,
    };

    if args.menu_args.menu {
        return screencast.rofi(args.menu_args.theme.as_ref());
    } else if let Some(area) = args.area_args.parse() {
        return match area {
            CaptureArea::Monitor => screencast.monitor(),
            CaptureArea::Selection => screencast.selection(),
        };
    }

    return Ok(());
}
