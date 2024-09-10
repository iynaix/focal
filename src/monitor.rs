use crate::SlurpGeom;

pub enum Rotation {
    Normal,
    Normal90,  // clockwise
    Normal270, // anti-clockwise
    Other,     // unimplemented
}

#[allow(clippy::module_name_repetitions)]
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
