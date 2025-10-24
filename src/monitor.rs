use crate::SlurpGeom;

#[derive(Debug, Clone)]
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
        (match self {
            Self::Normal => "",
            Self::Normal90 => "transpose=1",
            Self::Normal270 => "transpose=2",
            Self::Normal180 => "transpose=1,transpose=1",
            Self::Flipped => "hflip",
            Self::Flipped90 => "transpose=0",
            Self::Flipped270 => "transpose=3",
            Self::Flipped180 => "hflip,transpose=1,transpose=1",
        })
        .to_string()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct FocalMonitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub scale: f32,
    pub rotation: Rotation,
}

pub trait FocalMonitors {
    /// returns a vector of all monitors
    fn all(&self) -> Vec<FocalMonitor>;

    /// returns the focused monitor
    fn focused(&self) -> FocalMonitor;

    /// returns geometries of all visible (active) windows across all monitors
    fn window_geoms(&self) -> Vec<SlurpGeom>;

    /// total dimensions across all monitors
    fn total_dimensions(&self) -> (i32, i32) {
        let mut w = 0;
        let mut h = 0;
        for mon in self.all() {
            w = w.max(mon.x + mon.w);
            h = h.max(mon.y + mon.h);
        }

        (w, h)
    }
}
