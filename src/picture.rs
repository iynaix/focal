use std::{path::PathBuf, process::Stdio};

use crate::{Rofi, SlurpGeom};
use execute::{command, command_args, Execute};
use hyprland::{
    data::{Monitor, Monitors},
    shared::{HyprData, HyprDataActive},
};

#[derive(Default)]
struct Grim {
    monitor: String,
    geometry: String,
    output: PathBuf,
}

impl Grim {
    pub fn new(output: PathBuf) -> Self {
        Self {
            output,
            ..Default::default()
        }
    }

    pub fn geometry(mut self, geometry: &str) -> Self {
        self.geometry = geometry.to_string();
        self
    }

    pub fn monitor(mut self, monitor: &str) -> Self {
        self.monitor = monitor.to_string();
        self
    }

    pub fn capture(self, notify: bool) {
        let mut grim = command!("grim");

        if !self.monitor.is_empty() {
            grim.arg("-o").arg(self.monitor);
        }

        if !self.geometry.is_empty() {
            grim.arg("-g").arg(self.geometry);
        }

        grim.arg(&self.output)
            .execute()
            .expect("unable to execute grim");

        // show a notification
        if notify {
            command_args!("notify-send", "-t", "3000", "-a", "focal")
                .arg(format!("Screenshot captured to {}", self.output.display()))
                .arg("-i")
                .arg(&self.output)
                .execute()
                .expect("Failed to send screenshot notification");
        }
    }
}

pub struct Screenshot {
    pub delay: Option<u64>,
    pub edit: Option<String>,
    pub notify: bool,
    pub ocr: Option<String>,
    pub output: PathBuf,
}

impl Screenshot {
    fn capture(&self, monitor: &str, geometry: &str) {
        if self.ocr.is_none() {
            // copy the image file to clipboard
            command!("wl-copy")
                .arg("--type")
                .arg("text/uri-list")
                .execute_input(&format!("file://{}", self.output.display()))
                .expect("failed to copy image to clipboard");
        }

        // small delay before capture
        std::thread::sleep(std::time::Duration::from_millis(500));

        Grim::new(self.output.clone())
            .geometry(geometry)
            .monitor(monitor)
            .capture(self.notify);

        if self.ocr.is_some() {
            self.ocr();
        } else if self.edit.is_some() {
            self.edit();
        }
    }

    pub fn monitor(&self) {
        let focused = Monitor::get_active().expect("unable to get active monitor");
        self.capture(&focused.name, "");
    }

    pub fn selection(&self) {
        self.capture("", &SlurpGeom::prompt().to_string());
    }

    pub fn all(&self) {
        let mut w = 0;
        let mut h = 0;
        for mon in &Monitors::get().expect("unable to get monitors") {
            w = w.max(mon.x + i32::from(mon.width));
            h = h.max(mon.y + i32::from(mon.height));
        }

        self.capture("", &format!("0,0 {w}x{h}"));
    }

    fn edit(&self) {
        if let Some(prog) = &self.edit {
            if prog.ends_with("swappy") {
                command!("swappy")
                    .arg("--file")
                    .arg(self.output.clone())
                    .arg("--output-file")
                    .arg(self.output.clone())
                    .execute()
                    .expect("Failed to edit screenshot with swappy");
            } else {
                std::process::Command::new(prog)
                    .arg(self.output.clone())
                    .execute()
                    .expect("Failed to edit screenshot");
            }
        }
    }

    fn ocr(&self) {
        let mut cmd = command!("tesseract");
        cmd.arg(&self.output).arg("-");

        if let Some(lang) = &self.ocr {
            if !lang.is_empty() {
                cmd.arg("-l").arg(lang);
            }
        }

        let output = cmd
            .stdout(Stdio::piped())
            .execute_output()
            .expect("Failed to run tesseract");

        command!("wl-copy")
            .stdout(Stdio::piped())
            .execute_input(&output.stdout)
            .expect("unable to copy ocr text");
    }

    pub fn rofi(&mut self, theme: &Option<PathBuf>) {
        let mut opts = vec!["Selection", "Monitor"];

        // don't show all if single monitor
        if Monitors::get()
            .expect("unable to get monitors")
            .iter()
            .count()
            > 1
        {
            opts.push("All");
        };

        let mut rofi = Rofi::new(&opts);

        if let Some(theme) = theme {
            rofi = rofi.theme(theme.clone());
        }

        let (sel, exit_code) = rofi
            // for editing image
            .arg("-kb-custom-1")
            .arg("Alt-e")
            .arg("-mesg")
            .arg("Screenshots can be edited with Alt+e")
            .run();

        // no alt keycode selected, do not edit
        if exit_code != 10 {
            self.edit = None;
        }

        match sel.as_str() {
            "Selection" => self.selection(),
            "Monitor" => {
                std::thread::sleep(std::time::Duration::from_secs(Self::rofi_delay(theme)));
                self.monitor();
            }
            "All" => {
                std::thread::sleep(std::time::Duration::from_secs(Self::rofi_delay(theme)));
                self.all();
            }
            "" => {
                eprintln!("No capture selection was made.");
                std::process::exit(1);
            }
            _ => unimplemented!("Invalid rofi selection"),
        };
    }

    fn rofi_delay(theme: &Option<PathBuf>) -> u64 {
        let delay_options = ["0", "3", "5"];

        let mut rofi = Rofi::new(&delay_options);
        if let Some(theme) = theme {
            rofi = rofi.theme(theme.clone());
        }

        let (sel, _) = rofi.run();

        if sel.is_empty() {
            eprintln!("No delay selection was made.");
            std::process::exit(1);
        }

        sel.parse::<u64>().expect("Invalid delay specified")
    }
}
