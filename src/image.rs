use std::{path::PathBuf, process::Stdio};

use crate::{
    check_programs,
    cli::{Cli, CommonArgs},
    create_parent_dirs, iso8601_filename,
    monitor::FocalMonitors,
    rofi::RofiArgs,
    show_notification, Monitors, Rofi, SlurpGeom,
};
use clap::{ArgGroup, Args, CommandFactory, Subcommand, ValueEnum};
use execute::{command, Execute};

#[derive(Subcommand, ValueEnum, Debug, Clone)]
pub enum CaptureArea {
    Monitor,
    Selection,
    All,
}

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("area_shortcuts")
        .args(["area", "selection", "monitor", "all"])
        .multiple(false)
))]
pub struct AreaArgs {
    #[arg(
        short,
        long,
        visible_alias = "capture",
        value_enum,
        help = "Type of area to capture",
        long_help = "Type of area to capture\nShorthand aliases are also available"
    )]
    pub area: Option<CaptureArea>,

    #[arg(
        long,
        group = "area_shortcuts",
        help = "",
        long_help = "Shorthand for --area selection"
    )]
    pub selection: bool,

    #[arg(
        long,
        group = "area_shortcuts",
        help = "",
        long_help = "Shorthand for --area monitor"
    )]
    pub monitor: bool,

    #[arg(
        long,
        group = "area_shortcuts",
        help = "",
        long_help = "Shorthand for --area all"
    )]
    pub all: bool,
}

impl AreaArgs {
    pub fn parse(&self) -> Option<CaptureArea> {
        if self.selection {
            Some(CaptureArea::Selection)
        } else if self.monitor {
            Some(CaptureArea::Monitor)
        } else if self.all {
            Some(CaptureArea::All)
        } else {
            self.area.clone()
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("required_mode")
        .required(true)
        .multiple(false)
        .args(["rofi", "area", "selection", "monitor", "all"]),
))]
pub struct ImageArgs {
    #[command(flatten)]
    pub area_args: AreaArgs,

    #[command(flatten)]
    pub common_args: CommonArgs,

    #[command(flatten)]
    pub rofi_args: RofiArgs,

    #[arg(
        short,
        long,
        action,
        help = "Edit screenshot using COMMAND\nThe image path will be passed as $IMAGE",
        value_name = "COMMAND",
        conflicts_with = "ocr"
    )]
    pub edit: Option<String>,

    #[arg(
        long,
        num_args = 0..=1,
        value_name = "LANG",
        default_missing_value = "",
        action,
        help = "Runs OCR on the selected text",
        long_help = "Runs OCR on the selected text, defaulting to English\nSupported languages can be shown using 'tesseract --list-langs'",
        conflicts_with = "edit",
        hide = cfg!(not(feature = "ocr"))
    )]
    pub ocr: Option<String>,

    #[arg(
        name = "FILE",
        help = "Files are created in XDG_PICTURES_DIR/Screenshots if not specified"
    )]
    pub filename: Option<PathBuf>,
}

impl ImageArgs {
    pub fn required_programs(&self) -> Vec<&str> {
        let mut progs = vec!["grim"];

        if self.rofi_args.rofi {
            progs.push("rofi");
            progs.push("slurp");
        }

        if matches!(self.area_args.parse(), Some(CaptureArea::Selection)) {
            progs.push("slurp");
        }

        if self.ocr.is_some() {
            progs.push("tesseract");
        }

        progs
    }
}

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
            show_notification(
                &format!("Screenshot captured to {}", &self.output.display()),
                &self.output,
            );
        }
    }
}

pub struct Screenshot {
    pub delay: Option<u64>,
    pub no_rounded_windows: bool,
    pub edit: Option<String>,
    pub icons: bool,
    pub notify: bool,
    pub slurp: Option<String>,
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
            .capture(self.ocr.is_none() && self.notify);

        if self.ocr.is_some() {
            self.ocr();
        } else if self.edit.is_some() {
            self.edit();
        }
    }

    pub fn monitor(&self) {
        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));
        self.capture(&Monitors::focused().name, "");
    }

    pub fn selection(&self) {
        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));
        let (geom, is_window) = SlurpGeom::prompt(&self.slurp);

        let do_capture = || self.capture("", &geom.to_string());

        #[cfg(feature = "hyprland")]
        if is_window && self.no_rounded_windows {
            use hyprland::keyword::Keyword;

            if let Ok(Keyword {
                value: rounding, ..
            }) = Keyword::get("decoration:rounding")
            {
                Keyword::set("decoration:rounding", 0).expect("unable to disable rounding");
                do_capture();
                Keyword::set("decoration:rounding", rounding).expect("unable to restore rounding");
                return;
            }
        }

        do_capture();
    }

    pub fn all(&self) {
        let (w, h) = Monitors::total_dimensions();

        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));
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
        let mut opts = vec!["󰒉\tSelection", "󰍹\tMonitor", "󰍺\tAll"];

        // don't show "All" option if single monitor
        if Monitors::all().len() == 1 {
            opts.pop();
        };

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

        // only show edit message if an editor is provided
        let sel = if self.edit.is_some() {
            let (sel, exit_code) = rofi
                .arg("-kb-custom-1")
                .arg("Alt-e")
                .message("Screenshots can be edited with Alt+e")
                .run();

            // no alt keycode selected, do not edit
            if exit_code != 10 {
                self.edit = None;
            }

            sel
        } else {
            rofi.run().0
        };

        let sel = sel
            .split('\t')
            .collect::<Vec<&str>>()
            .pop()
            .unwrap_or_default();

        match sel {
            "Selection" => self.selection(),
            "Monitor" => {
                self.delay = Some(Self::rofi_delay(theme));
                self.monitor();
            }
            "All" => {
                self.delay = Some(Self::rofi_delay(theme));
                self.all();
            }
            "" => {
                eprintln!("No capture selection was made.");
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

pub fn main(args: ImageArgs) {
    dbg!(&args);

    if !cfg!(feature = "ocr") && args.ocr.is_some() {
        Cli::command()
            .error(
                clap::error::ErrorKind::UnknownArgument,
                "OCR support was not built in this version of focal.",
            )
            .exit()
    }

    // check if all required programs are installed
    check_programs(&args.required_programs());

    let fname = format!("{}.png", iso8601_filename());

    let output = if args.common_args.no_save {
        PathBuf::from(format!("/tmp/{fname}"))
    } else {
        create_parent_dirs(args.filename.unwrap_or_else(|| {
            dirs::picture_dir()
                .expect("could not get $XDG_PICTURES_DIR")
                .join(format!("Screenshots/{fname}"))
        }))
    };

    let mut screenshot = Screenshot {
        output,
        delay: args.common_args.delay,
        edit: args.edit,
        no_rounded_windows: args.common_args.no_rounded_windows,
        icons: !args.rofi_args.no_icons,
        notify: !args.common_args.no_notify,
        ocr: args.ocr,
        slurp: args.common_args.slurp,
    };

    if args.rofi_args.rofi {
        screenshot.rofi(&args.rofi_args.theme);
    } else if let Some(area) = args.area_args.parse() {
        match area {
            CaptureArea::Monitor => screenshot.monitor(),
            CaptureArea::Selection => screenshot.selection(),
            CaptureArea::All => screenshot.all(),
        }
    }
}
