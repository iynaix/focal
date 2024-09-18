use clap::Parser;
use focal::video::LockFile;
use std::{env, path::PathBuf, process::Command};

#[derive(Parser, Debug)]
#[command(
    name = "focal-waybar",
    about = "Updates waybar module with focal's recording status.",
    author,
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    #[arg(long, help = "Start / stop focal recording")]
    toggle: bool,

    #[arg(
        long,
        value_name = "N",
        default_value = "1",
        help = "Signal number to update module (SIGRTMIN+N), default is 1"
    )]
    signal: u8,

    #[arg(
        long,
        value_name = "MESSAGE",
        default_value = "REC",
        help = "Message to display in waybar module when recording"
    )]
    recording: String,

    #[arg(
        long,
        value_name = "MESSAGE",
        default_value = "",
        help = "Message to display in waybar module when not recording"
    )]
    stopped: String,

    // captures leftover arguments to be passed to `focal video`
    #[arg(
        allow_hyphen_values = true,
        num_args = 0..,
        help = "Additional arguments to pass to 'focal video'"
    )]
    pub focal_args: Vec<String>,
}

fn update_waybar(message: &str, args: &Cli) {
    println!("{message}");

    // waybar is wrapped on nixos
    let waybar_process = if PathBuf::from("/etc/NIXOS").exists() {
        ".waybar-wrapped"
    } else {
        "waybar"
    };

    Command::new("pkill")
        .arg("--signal")
        .arg(format!("SIGRTMIN+{}", args.signal))
        .arg(waybar_process)
        .output()
        .expect("Failed to execute pkill");
}

fn main() {
    let args = Cli::parse();

    let lock_exists = LockFile::exists();

    if args.toggle {
        if lock_exists {
            // stop the video
            Command::new("focal")
                .arg("video")
                .args(&args.focal_args)
                .output()
                .expect("Failed to execute focal");
            update_waybar(&args.stopped, &args);
        } else {
            // start recording
            update_waybar(&args.recording, &args);

            Command::new("focal")
                .arg("video")
                .args(&args.focal_args)
                .output()
                .expect("Failed to execute focal");
        }
        std::process::exit(0);
    }

    // no toggle, simple update
    if lock_exists {
        update_waybar(&args.recording, &args);
    } else {
        update_waybar(&args.stopped, &args);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focal_args() {
        let cmd = "focal-waybar --toggle --signal 1 --recording REC --stopped STOP --audio --rofi";
        let args = Cli::try_parse_from(cmd.split_whitespace());
        assert!(args.is_ok(), "{cmd}");
        if let Ok(args) = args {
            let msg = "test focal args";
            assert!(args.toggle, "{msg}");
            assert_eq!(args.signal, 1, "{msg}");
            assert_eq!(args.recording, "REC", "{msg}");
            assert_eq!(args.stopped, "STOP", "{msg}");
            assert_eq!(args.focal_args, ["--audio", "--rofi"], "{msg}");
        }
    }
}
