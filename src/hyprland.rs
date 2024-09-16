use hyprland::{
    data::{Clients, Monitor, Monitors, Transforms},
    shared::{HyprData, HyprDataActive},
};

use crate::{
    monitor::{FocalMonitor, FocalMonitors, Rotation},
    SlurpGeom,
};

fn to_focal_monitor(mon: &Monitor) -> FocalMonitor {
    FocalMonitor {
        name: mon.name.clone(),
        x: mon.x,
        y: mon.y,
        w: mon.width.into(),
        h: mon.height.into(),
        rotation: match mon.transform {
            Transforms::Normal => Rotation::Normal,
            Transforms::Normal90 => Rotation::Normal90,
            Transforms::Normal180 => Rotation::Normal180,
            Transforms::Normal270 => Rotation::Normal270,
            Transforms::Flipped => Rotation::Flipped,
            Transforms::Flipped90 => Rotation::Flipped90,
            Transforms::Flipped180 => Rotation::Flipped180,
            Transforms::Flipped270 => Rotation::Flipped270,
        },
    }
}

pub struct HyprMonitors;

impl FocalMonitors for HyprMonitors {
    fn all() -> Vec<FocalMonitor> {
        Monitors::get()
            .expect("unable to get monitors")
            .iter()
            .map(to_focal_monitor)
            .collect()
    }

    fn focused() -> FocalMonitor {
        to_focal_monitor(&Monitor::get_active().expect("unable to get active monitor"))
    }

    fn window_geoms() -> Vec<SlurpGeom> {
        let active_wksps: Vec<_> = Monitors::get()
            .expect("unable to get monitors")
            .iter()
            .map(|mon| mon.active_workspace.id)
            .collect();

        let windows = Clients::get().expect("unable to get clients");
        windows
            .iter()
            .filter(|&win| (active_wksps.contains(&win.workspace.id)))
            .map(|win| SlurpGeom {
                x: win.at.0.into(),
                y: win.at.1.into(),
                w: win.size.0.into(),
                h: win.size.1.into(),
            })
            .collect()
    }
}
