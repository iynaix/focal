use execute::{command, Execute};
use std::{fmt, process::Stdio};

use crate::{
    monitor::{FocalMonitors, Rotation},
    Monitors,
};

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy)]
pub struct SlurpGeom {
    pub w: i32,
    pub h: i32,
    pub x: i32,
    pub y: i32,
}

impl fmt::Display for SlurpGeom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{} {}x{}", self.x, self.y, self.w, self.h)
    }
}

impl std::str::FromStr for SlurpGeom {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(r"[,\sx]+").expect("Failed to create regex for slurp geom");

        let parts: Vec<_> = re
            .split(s)
            .map(|s| s.parse::<i32>().expect("Failed to parse slurp"))
            .collect();

        if parts.len() != 4 {
            return Err(ParseError::new("Slurp geom must have 4 parts"));
        }

        Ok(Self {
            x: parts[0],
            y: parts[1],
            w: parts[2],
            h: parts[3],
        })
    }
}

impl SlurpGeom {
    pub fn to_ffmpeg_geom(self) -> (String, String) {
        let Self { x, y, w, h } = self;

        let monitors = Monitors::all();
        let mon = monitors
            .iter()
            .find(|m| x >= m.x && x <= m.x + m.w && y >= m.y && y <= m.y + m.h)
            .unwrap_or_else(|| {
                panic!("No monitor found for slurp region");
            });

        // get coordinates relative to monitor
        let (x, y) = (x - mon.x, y - mon.y);
        let round2 = |n: i32| {
            if n % 2 == 1 {
                n - 1
            } else {
                n
            }
        };

        // h264 requires the width and height to be even
        let final_w = round2(h);
        let final_h = round2(w);

        let transpose = mon.rotation.ffmpeg_transpose();
        let filter = match mon.rotation {
            Rotation::Normal => format!("crop=w={w}:h={h}:x={x}:y={y}"),
            // clockwise
            Rotation::Normal90 => {
                let final_y = mon.w - x - w;
                let final_x = y;
                format!("crop=w={final_w}:h={final_h}:x={final_x}:y={final_y}")
            }
            // anti-clockwise
            Rotation::Normal270 => {
                let final_x = mon.w - y - h;
                let final_y = x;
                format!("crop=w={final_w}:h={final_h}:x={final_x}:y={final_y}")
            }
            _ => {
                unimplemented!("Unknown monitor transform");
            }
        };

        let filter = if transpose.is_empty() {
            filter
        } else {
            format!("{filter}, {transpose}")
        };

        (mon.name.clone(), filter)
    }

    #[cfg(feature = "hyprland")]
    pub fn disable_fade_animation() -> Option<String> {
        use hyprland::{
            data::{Animations, BezierIdent},
            shared::HyprData,
        };

        // remove fade animation
        let anims = Animations::get().expect("unable to get animations");
        anims.0.iter().find_map(|a| {
            (a.name == "fadeLayers").then(|| {
                let beizer = match &a.bezier {
                    BezierIdent::None => "",
                    BezierIdent::Default => "default",
                    BezierIdent::Specified(s) => s.as_str(),
                };
                format!(
                    "{},{},{},{}",
                    a.name,
                    std::convert::Into::<u8>::into(a.enabled),
                    a.speed,
                    beizer
                )
            })
        })
    }

    #[cfg(not(feature = "hyprland"))]
    pub const fn disable_fade_animation() -> Option<String> {
        None
    }

    #[cfg(feature = "hyprland")]
    pub fn reset_fade_animation(anim: &Option<String>) {
        use hyprland::keyword::Keyword;

        if let Some(anim) = anim {
            Keyword::set("animations", anim.clone()).expect("unable to set animations");
        }
    }

    #[cfg(not(feature = "hyprland"))]
    pub const fn reset_fade_animation(_anim: &Option<String>) {}

    pub fn prompt(slurp_args: &Option<String>) -> Self {
        let window_geoms = Monitors::window_geoms();

        let orig_fade_anim = Self::disable_fade_animation();

        let slurp_geoms = window_geoms
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");

        let mut slurp_cmd = command!("slurp");
        if let Some(slurp_args) = slurp_args {
            slurp_cmd.args(slurp_args.split_whitespace());
        } else {
            // sane slurp defaults
            slurp_cmd
                .arg("-c") // selection border
                .arg("#FFFFFFC0") // 0.75 opaque white
                .arg("-b") // background
                .arg("#000000C0") // 0.75 opaque black
                .arg("-B") // boxes
                .arg("#0000007F"); // 0.5 opaque black
        }

        let sel = slurp_cmd
            .stdout(Stdio::piped())
            .execute_input_output(&slurp_geoms)
            .map(|s| {
                std::str::from_utf8(&s.stdout).map_or_else(
                    |_| String::new(),
                    |s| s.strip_suffix("\n").unwrap_or_default().to_string(),
                )
            });

        // restore the original fade animation
        Self::reset_fade_animation(&orig_fade_anim);

        match sel {
            Ok(ref s) if s.is_empty() => {
                eprintln!("No slurp selection made");
                std::process::exit(1);
            }
            Err(_) => {
                eprintln!("Invalid slurp selection");
                std::process::exit(1);
            }
            Ok(sel) => window_geoms
                .into_iter()
                .find(|geom| geom.to_string() == sel)
                .unwrap_or_else(|| sel.parse().expect("Failed to parse slurp selection")),
        }
    }
}
