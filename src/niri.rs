use niri_ipc::{Output, Request, Response, Transform, socket::Socket};

use crate::{
    SlurpGeom,
    monitor::{FocalMonitor, FocalMonitors, Rotation},
};

#[allow(clippy::module_name_repetitions)]
pub struct NiriMonitors;

fn to_focal_monitor(mon: &Output) -> FocalMonitor {
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_possible_truncation)]
    mon.logical.map_or_else(
        || unimplemented!("Monitor {} is disabled!", mon.name),
        |logical| FocalMonitor {
            name: mon.name.clone(),
            x: logical.x,
            y: logical.y,
            w: logical.width as i32,
            h: logical.height as i32,
            scale: logical.scale as f32,
            rotation: match logical.transform {
                Transform::Normal => Rotation::Normal,
                Transform::_90 => Rotation::Normal90,
                Transform::_270 => Rotation::Normal270,
                Transform::_180 => Rotation::Normal180,
                Transform::Flipped => Rotation::Flipped,
                Transform::Flipped90 => Rotation::Flipped90,
                Transform::Flipped180 => Rotation::Flipped180,
                Transform::Flipped270 => Rotation::Flipped270,
            },
        },
    )
}

impl FocalMonitors for NiriMonitors {
    fn all() -> Vec<FocalMonitor> {
        let Ok(Response::Outputs(monitors)) = Socket::connect()
            .expect("failed to connect to niri socket")
            .send(Request::Outputs)
            .expect("failed to send Outputs request to niri")
        else {
            panic!("unexpected response from niri, should be Outputs");
        };

        monitors
            .iter()
            .filter_map(|(_, mon)| mon.logical.map(|_| mon))
            .map(to_focal_monitor)
            .collect()
    }

    fn focused() -> FocalMonitor {
        let Ok(Response::FocusedOutput(Some(monitor))) = Socket::connect()
            .expect("failed to connect to niri socket")
            .send(Request::Outputs)
            .expect("failed to send Outputs request to niri")
        else {
            panic!("unexpected response from niri, should be FocusedOutputs");
        };

        assert!(
            monitor.logical.is_some(),
            "Monitor {} is disabled!",
            monitor.name
        );

        to_focal_monitor(&monitor)
    }

    fn window_geoms() -> Vec<SlurpGeom> {
        // TODO: niri currently doesn't expose window geometries
        Vec::new()
    }
}
