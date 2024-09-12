use std::{path::PathBuf, process::Command};

#[cfg(feature = "hyprland")]
mod hyprland;
#[cfg(feature = "hyprland")]
use hyprland::HyprMonitors as Monitors;

#[cfg(feature = "sway")]
mod sway;
#[cfg(feature = "sway")]
use sway::SwayMonitors as Monitors;

pub mod cli;
pub mod image;
mod monitor;
pub mod rofi;
mod slurp;
pub mod video;

pub use image::Screenshot;
pub use rofi::Rofi;
pub use slurp::SlurpGeom;
pub use video::Screencast;

pub fn create_parent_dirs(path: PathBuf) -> PathBuf {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).expect("failed to create parent directories");
        }
    }

    path
}

pub fn iso8601_filename() -> String {
    chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub fn command_json<T: serde::de::DeserializeOwned>(cmd: &mut Command) -> T {
    let output = cmd.output().expect("Failed to execute command");
    let output_str = String::from_utf8(output.stdout).expect("unable to parse utf8 from command");

    serde_json::from_str(&output_str).expect("unable to parse json from command")
}

pub fn show_notification(body: &str, output: &PathBuf) {
    notify_rust::Notification::new()
        .body(body)
        .icon(&output.to_string_lossy())
        .appname("focal")
        .timeout(3000)
        .action("open", "open")
        .show()
        .expect("Failed to send notification")
        .wait_for_action(|action| {
            if action == "open" {
                std::process::Command::new("xdg-open")
                    .arg(output)
                    .spawn()
                    .expect("Failed to open file");
            }
        });
}
