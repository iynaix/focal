use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::{
    Monitors, Rofi, check_programs,
    cli::{
        Cli,
        image::{CaptureArea, ImageArgs},
    },
    create_parent_dirs, iso8601_filename,
    monitor::FocalMonitors,
    show_notification,
};
use arboard::Clipboard;
use clap::CommandFactory;
use execute::Execute;
use image::{ImageBuffer, Rgba};
use std::time::{Duration, Instant};

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
        let mut grim = Command::new("grim");

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
                Some(&self.output),
            );
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
pub struct Screenshot {
    pub delay: Option<u64>,
    pub no_rounded_windows: bool,
    pub freeze: bool,
    pub edit: Option<String>,
    pub icons: bool,
    pub notify: bool,
    pub slurp: Option<String>,
    pub ocr: Option<String>,
    pub output: PathBuf,
}

impl Screenshot {
    fn edit_or_ocr(&self) {
        if self.ocr.is_some() {
            self.ocr();
        } else {
            if self.edit.is_some() {
                self.edit();
            }

            // copying not needed for niri, since it already does that
            if cfg!(not(feature = "niri")) {
                let mut img = std::fs::File::open(&self.output).expect("failed to open image");
                Command::new("wl-copy")
                    .arg("--type")
                    .arg("image/png")
                    .execute_input_reader(&mut img)
                    .expect("failed to copy image to clipboard");
            }
        }
    }

    fn capture(&self, monitor: &str, geometry: &str) {
        // small delay before capture
        std::thread::sleep(std::time::Duration::from_millis(500));

        Grim::new(self.output.clone())
            .geometry(geometry)
            .monitor(monitor)
            .capture(self.ocr.is_none() && self.notify);

        self.edit_or_ocr();
    }

    // use niri's inbuilt screenshot
    #[cfg(feature = "niri")]
    pub fn monitor(&self) {
        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));

        std::process::Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("screenshot-screen")
            .spawn()
            .expect("unable to run `niri msg action screenshot-screen`")
            .wait()
            .expect("unable to wait for niri screenshot-screen");

        self.edit_or_ocr();
    }

    #[cfg(not(feature = "niri"))]
    pub fn monitor(&self) {
        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));
        self.capture(&Monitors::focused().name, "");
    }

    pub fn window(&self) {
        if cfg!(not(feature = "niri")) {
            self.selection();
        } else {
            use niri_ipc::{Action, Request, Response, Window, socket::Socket};

            std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));

            let mut socket = Socket::connect().expect("failed to connect to niri socket");
            let Ok(Response::PickedWindow(window)) = socket
                .send(Request::PickWindow)
                .expect("failed to send PickWindow request to niri")
            else {
                panic!("unexpected response from niri, should be PickWindow");
            };

            let Some(Window { id, .. }) = window else {
                eprintln!("No window was picked.");
                std::process::exit(1);
            };

            socket
                .send(Request::Action(Action::ScreenshotWindow {
                    id: Some(id),
                    write_to_disk: true,
                }))
                .expect("failed to send ScreenshotWindow request to niri")
                .expect("failed to screenshot window");
        }
    }

    #[cfg(feature = "niri")]
    pub fn selection(&self) {
        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));

        std::process::Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("screenshot")
            .spawn()
            .expect("unable to run `niri msg action screenshot`")
            .wait()
            .expect("unable to wait for niri screenshot");

        self.edit_or_ocr();
    }

    #[cfg(not(feature = "niri"))]
    pub fn selection(&self) {
        if self.freeze {
            Command::new("hyprpicker")
                .arg("-r")
                .arg("-z")
                .spawn()
                .expect("could not freeze screen")
                .wait()
                .expect("could not wait for freeze screen");
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));
        let (geom, is_window) = crate::SlurpGeom::prompt(self.slurp.as_deref());

        if self.freeze {
            Command::new("pkill")
                .arg("hyprpicker")
                .spawn()
                .expect("could not unfreeze screen")
                .wait()
                .expect("could not wait for unfreeze screen");
        }

        let do_capture = || {
            self.capture("", &geom.to_string());
        };

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
        if cfg!(feature = "niri") {
            unimplemented!("Capturing all screens with niri is not supported");
        }

        let (w, h) = Monitors::total_dimensions();

        std::thread::sleep(std::time::Duration::from_secs(self.delay.unwrap_or(0)));
        self.capture("", &format!("0,0 {w}x{h}"));
    }

    fn image_from_clipboard(&self) -> Option<PathBuf> {
        let mut clipboard = Clipboard::new().expect("failed to get clipboard");
        let start_time = Instant::now();
        let timeout = Duration::from_secs(1); // Set the maximum duration to 1 second
        let check_interval = Duration::from_millis(100); // Check every 100 milliseconds

        let mut image_found = false;

        while start_time.elapsed() < timeout {
            if let Ok(image_data) = clipboard.get_image() {
                // println!(
                //     "Image found on clipboard: {}x{} pixels (after {}ms)",
                //     image_data.width,
                //     image_data.height,
                //     start_time.elapsed().as_millis()
                // );

                // save the image
                #[allow(clippy::cast_possible_truncation)]
                if let Some(img) = ImageBuffer::<Rgba<u8>, _>::from_raw(
                    image_data.width as u32,
                    image_data.height as u32,
                    image_data.bytes.into_owned(),
                ) {
                    img.save(&self.output)
                        .expect("unable to save image to file");
                }

                image_found = true;
                break;
            }

            // Wait for the next check interval
            std::thread::sleep(check_interval);
        }

        image_found.then(|| self.output.clone())
    }

    fn edit(&self) {
        if cfg!(feature = "niri") && self.image_from_clipboard().is_none() {
            eprintln!("No image found on clipboard.");
            std::process::exit(1);
        }

        if let Some(prog) = &self.edit {
            if prog.ends_with("swappy") {
                Command::new("swappy")
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
        if cfg!(feature = "niri") && self.image_from_clipboard().is_none() {
            eprintln!("No image found on clipboard.");
            std::process::exit(1);
        }

        let mut cmd = Command::new("tesseract");
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

        let mut clipboard = Clipboard::new().expect("failed to get clipboard");
        clipboard
            .set_text(String::from_utf8_lossy(&output.stdout))
            .expect("unable to copy ocr text");

        if self.notify {
            if let Ok(copied_text) = std::str::from_utf8(&output.stdout) {
                show_notification(copied_text, None);
            }
        }
    }

    pub fn rofi(&mut self, theme: Option<&PathBuf>) {
        let mut opts = vec![
            "󰒉\tSelection",
            #[cfg(feature = "niri")]
            "󰖯\tWindow",
            "󰍹\tMonitor",
            "󰍺\tAll",
        ];

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
            "Window" => {
                self.delay = Some(Self::rofi_delay(theme));

                // TODO: use slurp to highlight geometry when selecting window?
                if cfg!(feature = "niri") {
                    self.window();
                } else {
                    self.selection();
                }
            }
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

pub fn main(args: ImageArgs) {
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
        freeze: args.freeze,
        edit: args.edit,
        no_rounded_windows: args.common_args.no_rounded_windows,
        icons: !args.rofi_args.no_icons,
        notify: !args.common_args.no_notify,
        ocr: args.ocr,
        slurp: args.common_args.slurp,
    };

    if args.rofi_args.rofi {
        screenshot.rofi(args.rofi_args.theme.as_ref());
    } else if let Some(area) = args.area_args.parse() {
        match area {
            CaptureArea::Monitor => screenshot.monitor(),
            CaptureArea::Window => {
                // TODO: use slurp to highlight geometry when selecting window?
                if cfg!(feature = "niri") {
                    screenshot.window();
                } else {
                    screenshot.selection();
                }
            }
            CaptureArea::Selection => screenshot.selection(),
            CaptureArea::All => screenshot.all(),
        }
    }
}
