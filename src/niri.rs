use color_eyre::eyre::{OptionExt, Result, eyre};
use niri_ipc::{Output, Request, Response, Transform, socket::Socket};

use crate::{
    SlurpGeom,
    monitor::{FocalMonitor, FocalMonitors, Rotation},
    screenshot::{FocalScreenshot, FocalScreenshotArgs},
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
    fn all(&self) -> Result<Vec<FocalMonitor>> {
        let Ok(Response::Outputs(monitors)) = Socket::connect()?.send(Request::Outputs)? else {
            panic!("unexpected response from niri, should be Outputs");
        };

        Ok(monitors
            .values()
            .filter_map(|mon| mon.logical.map(|_| mon))
            .map(to_focal_monitor)
            .collect())
    }

    fn focused(&self) -> Result<FocalMonitor> {
        let Ok(Response::FocusedOutput(Some(monitor))) =
            Socket::connect()?.send(Request::Outputs)?
        else {
            panic!("unexpected response from niri, should be FocusedOutputs");
        };

        assert!(
            monitor.logical.is_some(),
            "Monitor {} is disabled!",
            monitor.name
        );

        Ok(to_focal_monitor(&monitor))
    }

    fn window_geoms(&self) -> Result<Vec<SlurpGeom>> {
        // TODO: niri currently doesn't expose window geometries
        Ok(Vec::new())
    }
}

pub struct NiriScreenshot;

impl FocalScreenshot for NiriScreenshot {
    fn window(&self, args: &FocalScreenshotArgs) -> Result<()> {
        use niri_ipc::{Action, Request, Response, Window, socket::Socket};

        std::thread::sleep(std::time::Duration::from_secs(args.delay));

        let mut socket = Socket::connect()?;
        let Ok(Response::PickedWindow(window)) = socket.send(Request::PickWindow)? else {
            panic!("unexpected response from niri, should be PickWindow");
        };

        let Some(Window { id, .. }) = window else {
            eprintln!("No window was picked.");
            std::process::exit(1);
        };

        socket
            .send(Request::Action(Action::ScreenshotWindow {
                id: Some(id),
                path: Some(
                    args.output
                        .to_str()
                        .ok_or_eyre("invalid output path")?
                        .to_string(),
                ),
                show_pointer: false,
                write_to_disk: true,
            }))
            .map_err(|_| eyre!("failed to send ScreenshotWindow request to niri"))?
            .map_err(|_| eyre!("failed to screenshot window"))?;

        self.edit_or_ocr(args)
    }

    fn selection(&self, args: &FocalScreenshotArgs) -> Result<()> {
        use niri_ipc::{Action, Request, socket::Socket};

        std::thread::sleep(std::time::Duration::from_secs(args.delay));

        let mut socket = Socket::connect()?;
        socket
            .send(Request::Action(Action::Screenshot {
                path: Some(
                    args.output
                        .to_str()
                        .ok_or_eyre("invalid output path")?
                        .to_string(),
                ),
                show_pointer: false,
            }))
            .map_err(|_| eyre!("failed to send Screenshot request to niri"))?
            .map_err(|_| eyre!("failed to screenshot selection"))?;

        self.edit_or_ocr(args)
    }
}
