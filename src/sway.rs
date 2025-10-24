use std::process::Command;

use crate::{
    SlurpGeom, command_json,
    monitor::{FocalMonitor, FocalMonitors, Rotation},
};

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct GetOutput {
    pub name: String,
    pub rect: Rect,
    pub scale: f32,
    pub transform: String,
    pub focused: bool,
}

#[derive(Debug, Deserialize)]
struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Deserialize)]
struct GetTreeWindowNode {
    pub rect: Rect,
    pub nodes: Vec<GetTreeWindowNode>,
    // visible is only available in leaf (window) nodes
    pub visible: Option<bool>,
}

#[allow(clippy::used_underscore_items)]
impl GetTreeWindowNode {
    /// recursively collects all leaf nodes
    pub fn leaf_nodes(&self) -> Vec<&Self> {
        let mut leaf_nodes = Vec::new();
        self._leaf_nodes(&mut leaf_nodes);
        leaf_nodes
    }

    /// helper function for recursion
    fn _leaf_nodes<'a>(&'a self, leaf_nodes: &mut Vec<&'a Self>) {
        if self.nodes.is_empty() {
            leaf_nodes.push(self);
        } else {
            // recurse into child nodes
            for node in &self.nodes {
                node._leaf_nodes(leaf_nodes);
            }
        }
    }
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
        scale: mon.scale,
        rotation: match mon.transform.as_str() {
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
    }
}

fn window_geoms_cmd(cmd: &mut Command) -> Vec<SlurpGeom> {
    let tree: GetTreeWindowNode = command_json(cmd);

    tree.leaf_nodes()
        .iter()
        .filter(|&node| node.visible == Some(true))
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

impl FocalMonitors for SwayMonitors {
    fn all(&self) -> Vec<FocalMonitor> {
        let monitors: Vec<GetOutput> = command_json(
            Command::new("swaymsg")
                .arg("-t")
                .arg("get_outputs")
                .arg("--raw"),
        );

        monitors.iter().map(to_focal_monitor).collect()
    }

    fn focused(&self) -> FocalMonitor {
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

    fn window_geoms(&self) -> Vec<SlurpGeom> {
        window_geoms_cmd(
            Command::new("swaymsg")
                .arg("-t")
                .arg("get_tree")
                .arg("--raw"),
        )
    }
}
