use std::process::Command;

use crate::{
    SlurpGeom, command_json,
    monitor::{FocalMonitor, FocalMonitors, Rotation},
};

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HyprlandMonitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale: f32,
    pub transform: i32,
    pub focused: bool,
    pub active_workspace: HyprlandActiveWorkspace,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HyprlandActiveWorkspace {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HyprlandClient {
    pub workspace: HyprlandWorkspace,
    pub at: (i32, i32),
    pub size: (i32, i32),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HyprlandWorkspace {
    pub id: i64,
}

fn hyprland_monitors() -> Vec<HyprlandMonitor> {
    command_json(Command::new("hyprctl").arg("monitors").arg("-j"))
}

fn to_focal_monitor(mon: &HyprlandMonitor) -> FocalMonitor {
    FocalMonitor {
        name: mon.name.clone(),
        x: mon.x,
        y: mon.y,
        w: mon.width,
        h: mon.height,
        scale: mon.scale,
        rotation: match mon.transform {
            0 => Rotation::Normal,
            1 => Rotation::Normal90,
            2 => Rotation::Normal180,
            3 => Rotation::Normal270,
            4 => Rotation::Flipped,
            5 => Rotation::Flipped90,
            6 => Rotation::Flipped180,
            7 => Rotation::Flipped270,
            _ => unimplemented!("Invalid monitor transform: {}", mon.transform),
        },
    }
}

pub struct HyprMonitors;

impl FocalMonitors for HyprMonitors {
    fn all(&self) -> Vec<FocalMonitor> {
        hyprland_monitors().iter().map(to_focal_monitor).collect()
    }

    fn focused(&self) -> FocalMonitor {
        let monitors = hyprland_monitors();
        let curr_mon = monitors
            .iter()
            .find(|m| m.focused)
            .expect("unable to get focused monitor");

        to_focal_monitor(curr_mon)
    }

    fn window_geoms(&self) -> Vec<SlurpGeom> {
        let active_wksps: Vec<_> = hyprland_monitors()
            .iter()
            .map(|mon| mon.active_workspace.id)
            .collect();

        let all_clients: Vec<HyprlandClient> =
            command_json(Command::new("hyprctl").arg("clients").arg("-j"));

        all_clients
            .iter()
            .filter(|&win| active_wksps.contains(&win.workspace.id))
            .map(|win| SlurpGeom {
                x: win.at.0,
                y: win.at.1,
                w: win.size.0,
                h: win.size.1,
            })
            .collect()
    }
}
