use color_eyre::eyre::Result;
use std::{path::PathBuf, process::Command, sync::OnceLock};

mod grim;
mod hyprland;
mod mango;
mod niri;
mod screenshot;
mod sway;
mod umbriel;

pub mod cli;
pub mod image;
mod monitor;
pub mod noctalia;
pub mod rofi;
mod slurp;
pub mod video;
mod wf_recorder;

pub use noctalia::NoctaliaDmenu;
pub use rofi::Rofi;
pub use slurp::SlurpGeom;
pub use video::Screencast;

use crate::{monitor::FocalMonitors, screenshot::FocalScreenshot};

pub fn create_parent_dirs(path: PathBuf) -> Result<PathBuf> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    Ok(path)
}

pub fn iso8601_filename() -> String {
    chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub fn command_json<T: serde::de::DeserializeOwned>(cmd: &mut Command) -> Result<T> {
    let output = cmd.output()?;
    let output_str = String::from_utf8(output.stdout)?;

    Ok(serde_json::from_str(&output_str)?)
}

pub fn show_notification(body: &str, output: Option<&PathBuf>) -> Result<()> {
    let mut notification = notify_rust::Notification::new();

    notification.body(body);

    if let Some(output) = output {
        notification.icon(&output.to_string_lossy());
    }

    let notification = notification
        .appname("focal")
        .timeout(3000)
        .action("open", "open")
        .show()?;

    if let Some(output) = output {
        notification.wait_for_action(|action| {
            if action == "open"
                && let Err(e) = std::process::Command::new("xdg-open")
                    .arg(output)
                    .spawn()
                    .and_then(|mut child| child.wait())
            {
                eprintln!("Failed to open file: {e}");
            }
        });
    }

    Ok(())
}

/// check if all required programs are installed
pub fn check_programs(progs: &[&str]) {
    let mut all_progs = std::collections::HashSet::from(["wl-copy", "xdg-open"]);

    all_progs.extend(progs);

    let not_found: Vec<_> = all_progs
        .into_iter()
        .filter(|prog| which::which(prog).is_err())
        .collect();

    if !not_found.is_empty() {
        eprintln!(
            "The following programs are required but not installed: {}",
            not_found.join(", ")
        );
        std::process::exit(1);
    }
}

pub fn is_hyprland() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default() == "Hyprland"
}

pub fn is_niri() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default() == "niri"
}

pub fn is_sway() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default() == "sway"
}

pub fn is_mango() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default() == "mango"
}

pub fn is_umbriel() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default() == "umbriel"
}

pub fn is_noctalia() -> bool {
    false

    // static CACHE: OnceLock<bool> = OnceLock::new();
    // *CACHE.get_or_init(|| which::which("noctalia").is_ok())
}

pub fn focal_monitor() -> Box<dyn FocalMonitors> {
    match std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .as_str()
    {
        "Hyprland" => Box::new(hyprland::HyprMonitors),
        "niri" => Box::new(niri::NiriMonitors),
        "sway" => Box::new(sway::SwayMonitors),
        "mango" => Box::new(mango::MangoMonitors),
        "umbriel" => Box::new(umbriel::UmbrielMonitors),
        _ => unimplemented!("Unsupported desktop environment"),
    }
}

pub fn focal_screenshot() -> Box<dyn FocalScreenshot> {
    if is_noctalia() {
        return Box::new(noctalia::NoctaliaScreenshot);
    }

    match std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .as_str()
    {
        "Hyprland" => Box::new(hyprland::HyprScreenshot),
        "niri" => Box::new(niri::NiriScreenshot),
        "sway" => Box::new(sway::SwayScreenshot),
        "mango" => Box::new(mango::MangoScreenshot),
        "umbriel" => Box::new(umbriel::UmbrielScreenshot),
        _ => unimplemented!("Unsupported desktop environment"),
    }
}
