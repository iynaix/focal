use color_eyre::eyre::Result;
use libwayshot::LogicalRegion;
use std::process::Command;

use crate::{
    SlurpGeom, command_json, focal_monitor,
    monitor::{FocalMonitor, FocalMonitors, Rotation},
    screenshot::{FocalScreenshot, FocalScreenshotArgs},
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

impl FocalScreenshot for UmbrielScreenshot {
    fn monitor(&self, args: &FocalScreenshotArgs) -> Result<()> {
        use libwayshot::WayshotConnection;

        std::thread::sleep(std::time::Duration::from_secs(args.delay));

        let curr_mon = focal_monitor().focused()?.name;

        let conn = WayshotConnection::new()?;
        let outputs = conn.get_all_outputs();

        let curr_output = outputs
            .iter()
            .find(|output| output.name == curr_mon)
            .unwrap_or_else(|| {
                eprintln!("Unable to get current output");
                std::process::exit(1);
            });

        let img = conn.screenshot_single_output(curr_output, false)?;
        img.save("/tmp/monitor.png")?;

        self.edit_or_ocr(args)
    }

    fn window(&self, args: &FocalScreenshotArgs) -> Result<()> {
        use libwayshot::WayshotConnection;

        std::thread::sleep(std::time::Duration::from_secs(args.delay));

        let conn = WayshotConnection::new()?;
        let windows = conn.get_all_toplevels();

        /*
        umbriel focused-window --json
        {"active":true,"app_id":"kitty","border_effect":{"name":"","overlay":"","pool":"","source":"default","suppressed":false},"column":1,"content_type":"none","floating":false,"focused":true,"fullscreen":false,"h":1390,"id":"7c72ce35dd6cb56cc7e59d832450838e","maximized":false,"maximized_to_edges":false,"minimizable":false,"minimized":false,"output":"DP-1","parent":"","pid":3366,"pinned":false,"row":0,"scratchpad":"","snap":null,"tab_hidden":false,"tab_index":-1,"tabbed":false,"title":"umbriel focused-wind ~","title_bar":false,"urgent":false,"visible":true,"w":1688,"window_effect":{"name":"","pool":"","source":"default","suppressed":false},"workspace":"DP-1:2","x":3166,"xdg_tag":"","xwayland":false,"y":1120}
        */

        let curr_win = windows
            .iter()
            .find(|win| win.identifier == "7c72ce35dd6cb56cc7e59d832450838e")
            .unwrap_or_else(|| {
                eprintln!("Unable to get current window");
                std::process::exit(1);
            });

        let img = conn.screenshot_toplevel(curr_win, false)?;
        img.save("/tmp/window.png")?;

        self.edit_or_ocr(args)
    }

    fn selection(&self, args: &FocalScreenshotArgs) -> Result<()> {
        use libwayshot::WayshotConnection;

        std::thread::sleep(std::time::Duration::from_secs(args.delay));

        let geom = SlurpGeom::prompt(None)?;

        let conn = WayshotConnection::new()?;
        let img = conn.screenshot(geom.to_logical_region(), false)?;
        img.save("/tmp/selection.png")?;

        self.edit_or_ocr(args)
    }
}
