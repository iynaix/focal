use crate::SlurpGeom;

#[derive(Debug)]
pub enum Rotation {
    Normal,
    /// Clockwise
    Normal90,
    /// 180 degrees
    Normal180,
    /// Anti-clockwise
    Normal270,
    /// Flipped
    Flipped,
    /// Flipped and rotated clockwise
    Flipped90,
    /// Flipped and rotated 180 degrees
    Flipped180,
    /// Flipped and rotated anti-clockwise
    Flipped270,
}

impl Rotation {
    pub fn ffmpeg_transpose(&self) -> String {
        match self {
            Self::Normal => String::new(),
            // clockwise
            Self::Normal90 => "transpose=1".into(),
            // anti-clockwise
            Self::Normal270 => "transpose=2".into(),
            _ => {
                unimplemented!("Unknown monitor transform");
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct FocalMonitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub rotation: Rotation,
}

pub trait FocalMonitors {
    /// returns a vector of all monitors
    fn all() -> Vec<FocalMonitor>
    where
        Self: std::marker::Sized;

    /// returns the focused monitor
    fn focused() -> FocalMonitor;

    /// returns geometries of all visible (active) windows across all monitors
    fn window_geoms() -> Vec<SlurpGeom>;
}
