use execute::{command_args, Execute};
use hyprland::{
    data::{Animations, BezierIdent, Clients, Monitors, Transforms},
    keyword::Keyword,
    shared::HyprData,
};
use std::{fmt, process::Stdio};

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
    w: i32,
    h: i32,
    x: i32,
    y: i32,
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

        let monitors = Monitors::get().expect("unable to get monitors");
        let mon = monitors
            .iter()
            .find(|m| {
                x >= m.x
                    && x <= m.x + i32::from(m.width)
                    && y >= m.y
                    && y <= m.y + i32::from(m.height)
            })
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

        let filter = match mon.transform {
            Transforms::Normal => format!("crop=w={w}:h={h}:x={x}:y={y}"),
            // clockwise
            Transforms::Normal90 => {
                let final_y = i32::from(mon.width) - x - w;
                let final_x = y;
                format!("crop=w={final_w}:h={final_h}:x={final_x}:y={final_y}, transpose=1")
            }
            // anti-clockwise
            Transforms::Normal270 => {
                let final_x = i32::from(mon.width) - y - h;
                let final_y = x;
                format!("crop=w={final_w}:h={final_h}:x={final_x}:y={final_y}, transpose=2")
            }
            _ => {
                unimplemented!("Unknown monitor transform");
            }
        };

        (mon.name.clone(), filter)
    }

    pub fn disable_fade_animation() -> Option<String> {
        // remove fade animation
        let anims = Animations::get().expect("unable to get animations");
        anims.0.iter().find_map(|a| {
            if a.name == "fadeLayers" {
                let beizer = match &a.bezier {
                    BezierIdent::None => "",
                    BezierIdent::Default => "default",
                    BezierIdent::Specified(s) => s.as_str(),
                };
                return Some(format!(
                    "{},{},{},{}",
                    a.name,
                    std::convert::Into::<u8>::into(a.enabled),
                    a.speed,
                    beizer
                ));
            }
            None
        })
    }

    pub fn reset_fade_animation(anim: &Option<String>) {
        if let Some(anim) = anim {
            Keyword::set("animations", anim.clone()).expect("unable to set animations");
        }
    }

    pub fn prompt() -> Self {
        let active_wksps: Vec<_> = Monitors::get()
            .expect("unable to get monitors")
            .iter()
            .map(|mon| mon.active_workspace.id)
            .collect();

        let windows = Clients::get().expect("unable to get clients");
        let window_geoms: Vec<_> = windows
            .iter()
            .filter_map(|win| {
                if active_wksps.contains(&win.workspace.id) {
                    return Some(Self {
                        x: win.at.0.into(),
                        y: win.at.1.into(),
                        w: win.size.0.into(),
                        h: win.size.1.into(),
                    });
                }
                None
            })
            .collect();

        let orig_fade_anim = Self::disable_fade_animation();

        let slurp_geoms = window_geoms
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");

        let sel = command_args!("slurp")
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
