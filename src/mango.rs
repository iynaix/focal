use crate::{
    SlurpGeom, command_json,
    monitor::{FocalMonitor, FocalMonitors, Rotation},
    screenshot::FocalScreenshot,
};
use color_eyre::eyre::{OptionExt, Result};
use std::process::Command;

use serde_derive::Deserialize;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct MangoAllMonitors {
    pub monitors: Vec<MangoMonitor>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct MangoMonitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale: f32,
    pub active: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MangoAllClients {
    pub clients: Vec<Client>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Client {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_visible: bool,
}

#[allow(clippy::module_name_repetitions)]
pub struct MangoMonitors;

fn to_focal_monitor(mon: &MangoMonitor) -> FocalMonitor {
    FocalMonitor {
        name: mon.name.clone(),
        x: mon.x,
        y: mon.y,
        w: mon.width,
        h: mon.height,
        scale: mon.scale,
        // TODO: rotation not included????
        rotation: Rotation::Normal,
    }
}

fn mango_all_monitors() -> Result<Vec<MangoMonitor>> {
    let resp: MangoAllMonitors = command_json(Command::new("mmsg").arg("get").arg("all-monitors"))?;

    Ok(resp.monitors)
}

impl FocalMonitors for MangoMonitors {
    fn all(&self) -> Result<Vec<FocalMonitor>> {
        mango_all_monitors().map(|monitors| monitors.iter().map(to_focal_monitor).collect())
    }

    fn focused(&self) -> Result<FocalMonitor> {
        mango_all_monitors()?
            .iter()
            .find(|mon| mon.active)
            .map(to_focal_monitor)
            .ok_or_eyre("No focused monitor")
    }

    fn window_geoms(&self) -> Result<Vec<SlurpGeom>> {
        let resp: MangoAllClients =
            command_json(Command::new("mmsg").arg("get").arg("all-clients"))?;

        Ok(resp
            .clients
            .iter()
            .filter(|win| win.is_visible)
            .map(|win| SlurpGeom {
                w: win.width,
                h: win.height,
                x: win.x,
                y: win.y,
            })
            .collect())
    }
}

pub struct MangoScreenshot;

impl FocalScreenshot for MangoScreenshot {}
