use crate::{
    SlurpGeom, command_json,
    monitor::{FocalMonitor, FocalMonitors, Rotation},
};
use std::process::{Command, Stdio};

use serde_derive::Deserialize;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WlrMonitor {
    pub name: String,
    pub enabled: bool,
    pub modes: Vec<Mode>,
    pub position: Position,
    pub transform: String,
    pub scale: f32,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mode {
    pub width: i32,
    pub height: i32,
    pub current: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[allow(clippy::module_name_repetitions)]
pub struct MangoMonitors;

fn to_focal_monitor(mon: &WlrMonitor) -> FocalMonitor {
    let mode = mon
        .modes
        .iter()
        .find(|mode| mode.current)
        .unwrap_or_else(|| {
            eprintln!("Monitor {} has no current mode!", mon.name);
            std::process::exit(1);
        });

    FocalMonitor {
        name: mon.name.clone(),
        x: mon.position.x,
        y: mon.position.y,
        w: mode.width,
        h: mode.height,
        scale: mon.scale,
        rotation: match mon.transform.as_str() {
            "normal" => Rotation::Normal,
            "90" => Rotation::Normal90,
            "180" => Rotation::Normal180,
            "270" => Rotation::Normal270,
            "flipped" => Rotation::Flipped,
            "flipped-90" => Rotation::Flipped90,
            "flipped-180" => Rotation::Flipped180,
            "flipped-270" => Rotation::Flipped270,
            _ => unimplemented!("Invalid monitor transform"),
        },
    }
}

impl FocalMonitors for MangoMonitors {
    fn all(&self) -> Vec<FocalMonitor> {
        let monitors: Vec<WlrMonitor> = command_json(Command::new("wlr-randr").arg("--json"));

        monitors
            .iter()
            .filter(|mon| mon.enabled)
            .map(to_focal_monitor)
            .collect()
    }

    fn focused(&self) -> FocalMonitor {
        let output = Command::new("mmsg")
            .arg("-g")
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to execute mmsg -g");

        let stdout = String::from_utf8_lossy(&output.stdout);

        let sel_mon = stdout
            .lines()
            .find(|line| line.ends_with("selmon 1"))
            .map_or_else(
                || {
                    eprintln!("Unable to get focused monitor.");
                    std::process::exit(1);
                },
                |line| line.split(' ').next().expect("invalid selmon").to_string(),
            );

        self.all()
            .iter()
            .find(|mon| mon.name == sel_mon)
            .unwrap_or_else(|| {
                eprintln!("Unable to get focused monitor.");
                std::process::exit(1);
            })
            .clone()
    }

    fn window_geoms(&self) -> Vec<SlurpGeom> {
        // TODO: mango currently doesn't expose window geometries, see:
        // https://github.com/DreamMaoMao/mangowc/issues/418
        Vec::new()
    }
}
