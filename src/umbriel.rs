use color_eyre::eyre::Result;
use std::process::Command;

use crate::{
    SlurpGeom, command_json,
    monitor::{FocalMonitor, FocalMonitors, Rotation},
    screenshot::FocalScreenshot,
};

use color_eyre::eyre::OptionExt;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UmbrielMonitor {
    pub enabled: bool,
    pub modes: Vec<UmbrielMonitorMode>,
    pub name: String,
    pub position: UmbrielMonitorPosition,
    pub scale: f32,
    pub transform: String,
}

#[derive(Debug, Deserialize)]
pub struct UmbrielMonitorMode {
    pub current: bool,
    pub height: i32,
    pub width: i32,
}

#[derive(Debug, Deserialize)]
pub struct UmbrielMonitorPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Deserialize)]
pub struct UmbrielWorkspace {
    pub index: i32,
    pub active: bool,
    pub focused: bool,
    pub output: String,
}

#[derive(Debug, Deserialize)]
pub struct UmbrielWindow {
    pub w: i32,
    pub h: i32,
    pub workspace: String,
    pub x: i32,
    pub y: i32,
}

#[allow(clippy::module_name_repetitions)]
pub struct UmbrielMonitors;

impl FocalMonitors for UmbrielMonitors {
    fn all(&self) -> Result<Vec<FocalMonitor>> {
        let mons: Vec<UmbrielMonitor> =
            command_json(Command::new("umbriel").arg("outputs").arg("--json"))?;

        Ok(mons
            .into_iter()
            .filter_map(|mon| {
                if !mon.enabled {
                    return None;
                }

                // ignore if no current mode
                let mode = mon.modes.iter().find(|mode| mode.current)?;

                Some(FocalMonitor {
                    name: mon.name,
                    x: mon.position.x,
                    y: mon.position.y,
                    w: mode.width,
                    h: mode.height,
                    scale: mon.scale,
                    rotation: match mon.transform.to_lowercase().as_str() {
                        "normal" => Rotation::Normal,
                        "90" => Rotation::Normal90,
                        "270" => Rotation::Normal270,
                        "180" => Rotation::Normal180,
                        "flipped" => Rotation::Flipped,
                        "flipped-90" => Rotation::Flipped90,
                        "flipped-180" => Rotation::Flipped180,
                        "flipped-270" => Rotation::Flipped270,
                        _ => unimplemented!("Invalid monitor transform"),
                    },
                })
            })
            .collect())
    }

    fn focused(&self) -> Result<FocalMonitor> {
        let wksps: Vec<UmbrielWorkspace> =
            command_json(Command::new("umbriel").arg("workspaces").arg("--json"))?;

        let focused_mon = wksps
            .into_iter()
            .find_map(|wksp| {
                if wksp.active && wksp.focused {
                    Some(wksp.output)
                } else {
                    None
                }
            })
            .ok_or_eyre("unable get focused workspace")?;

        self.all()?
            .into_iter()
            .find(|mon| mon.name == focused_mon)
            .ok_or_eyre("unable to get focused monitor")
    }

    fn window_geoms(&self) -> Result<Vec<SlurpGeom>> {
        let wksps: Vec<UmbrielWorkspace> =
            command_json(Command::new("umbriel").arg("workspaces").arg("--json"))?;

        let active_wksps: std::collections::HashSet<_> = wksps
            .into_iter()
            .filter_map(|wksp| {
                if wksp.active {
                    Some(format!("{}:{}", wksp.output, wksp.index))
                } else {
                    None
                }
            })
            .collect();

        let windows: Vec<UmbrielWindow> =
            command_json(Command::new("umbriel").arg("windows").arg("--json"))?;

        Ok(windows
            .iter()
            .filter_map(|win| {
                if active_wksps.contains(&win.workspace) {
                    Some(SlurpGeom {
                        w: win.w,
                        h: win.h,
                        x: win.x,
                        y: win.y,
                    })
                } else {
                    None
                }
            })
            .collect())
    }
}

pub struct UmbrielScreenshot;

impl FocalScreenshot for UmbrielScreenshot {}
