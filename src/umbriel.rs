use std::process::Command;

use crate::{
    SlurpGeom, command_json,
    monitor::{FocalMonitor, FocalMonitors, Rotation},
};

use serde_derive::Deserialize;

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
    fn all(&self) -> Vec<FocalMonitor> {
        let cmd = std::process::Command::new("umbriel")
            .arg("outputs")
            .output()
            .expect("Failed to execute umbriel outputs");
        let output = String::from_utf8(cmd.stdout).expect("unable to parse utf8 from command");

        output
            .split("\n\n")
            .filter_map(|mon| {
                let mut focal_mon = FocalMonitor::default();

                for line in mon.split("\n") {
                    match line.split_once(":") {
                        Some((k, v)) => {
                            let v = v.trim();

                            match k.trim() {
                                "Enabled" => {
                                    // skip disabled monitors
                                    if v.to_lowercase() != "yes" {
                                        return None;
                                    }
                                }
                                "Position" => {
                                    let pos = v.split_once(",").expect("Invalid position");
                                    focal_mon.x = pos.0.parse().expect("Invalid position x");
                                    focal_mon.y = pos.1.parse().expect("Invalid position y");
                                }
                                "Transform" => {
                                    focal_mon.rotation = match v.to_lowercase().as_str() {
                                        "normal" => Rotation::Normal,
                                        "90" => Rotation::Normal90,
                                        "270" => Rotation::Normal270,
                                        "180" => Rotation::Normal180,
                                        "flipped" => Rotation::Flipped,
                                        "flipped-90" => Rotation::Flipped90,
                                        "flipped-180" => Rotation::Flipped180,
                                        "flipped-270" => Rotation::Flipped270,
                                        _ => unimplemented!("Invalid monitor transform"),
                                    };
                                }
                                "Scale" => {
                                    focal_mon.scale = v.parse().expect("Invalid scale");
                                }
                                _ => {}
                            };
                        }
                        // modes have no key
                        None => {
                            if line.contains(" Hz") {
                                if line.contains("current") {
                                    let mode = line
                                        .trim()
                                        .split_once(" ")
                                        .expect("Invalid mode")
                                        .0
                                        .split_once("x")
                                        .expect("Invalid mode");

                                    focal_mon.w = mode.0.parse().expect("Invalid width");
                                    focal_mon.h = mode.1.parse().expect("Invalid height");
                                }
                            } else if line != "" {
                                focal_mon.name = line
                                    .trim()
                                    .split_once(" ")
                                    .expect("Unable to parse monitor name")
                                    .0
                                    .trim()
                                    .to_string();
                            }
                        }
                    }
                }

                Some(focal_mon)
            })
            .collect()
    }

    fn focused(&self) -> FocalMonitor {
        let wksps: Vec<UmbrielWorkspace> =
            command_json(Command::new("umbriel").arg("workspaces").arg("--json"));

        let focused_mon = wksps
            .into_iter()
            .find_map(|wksp| {
                if wksp.active && wksp.focused {
                    Some(wksp.output)
                } else {
                    None
                }
            })
            .expect("unable get focused workspace");

        self.all()
            .into_iter()
            .find(|mon| mon.name == focused_mon)
            .expect("unable to get focused monitor")
    }

    fn window_geoms(&self) -> Vec<SlurpGeom> {
        let wksps: Vec<UmbrielWorkspace> =
            command_json(Command::new("umbriel").arg("workspaces").arg("--json"));

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
            command_json(Command::new("umbriel").arg("windows").arg("--json"));

        windows
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
            .collect()
    }
}
