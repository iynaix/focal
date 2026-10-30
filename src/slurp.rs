use color_eyre::eyre::Result;
use execute::Execute;
use libwayshot::{
    LogicalRegion, Size,
    region::{Position, Region},
};
use std::{
    fmt,
    process::{Command, Stdio},
};

use crate::focal_monitor;

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

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, Clone, Copy)]
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
        let re = regex::Regex::new(r"[,\sx]+")
            .map_err(|_| ParseError::new("could not create slurp regex"))?;

        let parts: Vec<_> = re.split(s).filter_map(|s| s.parse::<i32>().ok()).collect();

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

const fn round2(n: i32) -> i32 {
    if n % 2 == 1 { n - 1 } else { n }
}

impl SlurpGeom {
    pub fn to_ffmpeg_geom(self) -> Result<(String, String)> {
        let Self { x, y, w, h } = self;

        let monitors = focal_monitor().all()?;
        let mon = monitors
            .iter()
            .find(|m| x >= m.x && x <= m.x + m.w && y >= m.y && y <= m.y + m.h)
            .unwrap_or_else(|| {
                panic!("No monitor found for slurp region");
            });

        // get coordinates relative to monitor
        let (mut w, mut h) = (w, h);
        let (mut x, mut y) = (x - mon.x, y - mon.y);

        // handle monitor scaling
        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::cast_possible_truncation)]
        // mon.scale != 1.0
        if (mon.scale - 1.0).abs() > f32::EPSILON {
            x = (x as f32 * mon.scale).round() as i32;
            y = (y as f32 * mon.scale).round() as i32;
            w = (w as f32 * mon.scale).round() as i32;
            h = (h as f32 * mon.scale).round() as i32;
        }

        // h264 requires the width and height to be even
        w = round2(w);
        h = round2(h);

        let transpose = mon.rotation.ffmpeg_transpose();
        let filter = format!(
            "{}crop=w={w}:h={h}:x={x}:y={y}",
            if transpose.is_empty() {
                String::new()
            } else {
                format!("{transpose}, ")
            }
        );

        Ok((mon.name.clone(), filter))
    }

    pub const fn to_logical_region(self) -> LogicalRegion {
        LogicalRegion {
            inner: Region {
                position: Position {
                    x: self.x,
                    y: self.y,
                },
                size: Size {
                    width: self.w.cast_unsigned(),
                    height: self.h.cast_unsigned(),
                },
            },
        }
    }

    /// returns the selected geometry and if a window was selected
    pub fn prompt(slurp_args: Option<&str>) -> Result<Self> {
        let window_geoms = focal_monitor().window_geoms()?;
        let slurp_geoms = window_geoms
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");

        let mut slurp_cmd = Command::new("slurp");
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

        match sel {
            Ok(ref s) if s.is_empty() => {
                eprintln!("No slurp selection made");
                std::process::exit(1);
            }
            Err(_) => {
                eprintln!("Invalid slurp selection");
                std::process::exit(1);
            }
            Ok(sel) => {
                if let Some(geom) = window_geoms.into_iter().find(|g| g.to_string() == sel) {
                    Ok(geom)
                } else {
                    let parsed = sel.parse()?;
                    Ok(parsed)
                }
            }
        }
    }
}
