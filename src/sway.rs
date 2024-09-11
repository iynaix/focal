use std::process::Command;

use crate::{
    command_json,
    monitor::{FocalMonitor, FocalMonitors, Rotation},
    SlurpGeom,
};

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetOutput {
    pub name: String,
    pub rect: Rect,
    pub transform: String,
    pub focused: bool,
}

#[derive(Debug, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetTree {
    pub nodes: Vec<GetTreeMonitorNode>,
}

#[derive(Debug, Deserialize)]
pub struct GetTreeMonitorNode {
    pub nodes: Vec<GetTreeWorkspaceNode>,
}

#[derive(Debug, Deserialize)]
pub struct GetTreeWorkspaceNode {
    pub nodes: Vec<GetTreeWindowNode>,
}

#[derive(Debug, Deserialize)]
pub struct GetTreeWindowNode {
    pub rect: Rect,
    pub visible: bool,
}

#[allow(clippy::module_name_repetitions)]
pub struct SwayMonitors;

fn to_focal_monitor(mon: &GetOutput) -> FocalMonitor {
    FocalMonitor {
        name: mon.name.clone(),
        x: mon.rect.x,
        y: mon.rect.y,
        w: mon.rect.width,
        h: mon.rect.height,
        rotation: match mon.transform.as_str() {
            "normal" => Rotation::Normal,
            "90" => Rotation::Normal90,
            "270" => Rotation::Normal270,
            _ => Rotation::Other,
        },
    }
}

impl FocalMonitors for SwayMonitors {
    fn all() -> Vec<FocalMonitor> {
        let monitors: Vec<GetOutput> = command_json(
            Command::new("swaymsg")
                .arg("-t")
                .arg("get_outputs")
                .arg("--raw"),
        );

        monitors.iter().map(to_focal_monitor).collect()
    }

    fn focused() -> FocalMonitor {
        let monitors: Vec<GetOutput> = command_json(
            Command::new("swaymsg")
                .arg("-t")
                .arg("get_outputs")
                .arg("--raw"),
        );

        monitors
            .iter()
            .find_map(|m| m.focused.then_some(to_focal_monitor(m)))
            .expect("no focused monitor")
    }

    fn window_geoms() -> Vec<SlurpGeom> {
        let tree: GetTree = command_json(
            Command::new("swaymsg")
                .arg("-t")
                .arg("get_tree")
                .arg("--raw"),
        );

        tree.nodes
            .iter()
            .flat_map(|mon_node| mon_node.nodes.iter())
            .flat_map(|wksp_node| wksp_node.nodes.iter())
            // only want visible windows
            .filter(|&win_node| win_node.visible)
            .map(|win_node| {
                let rect = &win_node.rect;
                SlurpGeom {
                    x: rect.x,
                    y: rect.y,
                    w: rect.width,
                    h: rect.height,
                }
            })
            .collect()
    }
}
