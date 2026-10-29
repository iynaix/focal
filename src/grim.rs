use color_eyre::eyre::Result;
use execute::Execute;
use std::{path::PathBuf, process::Command};

#[derive(Default)]
pub struct Grim {
    monitor: String,
    geometry: String,
    output: PathBuf,
}

impl Grim {
    pub fn new(output: PathBuf) -> Self {
        Self {
            output,
            ..Default::default()
        }
    }

    pub fn geometry(mut self, geometry: &str) -> Self {
        self.geometry = geometry.to_string();
        self
    }

    pub fn monitor(mut self, monitor: &str) -> Self {
        self.monitor = monitor.to_string();
        self
    }

    pub fn capture(&self) -> Result<()> {
        let mut grim = Command::new("grim");

        if !self.monitor.is_empty() {
            grim.arg("-o").arg(&self.monitor);
        }

        if !self.geometry.is_empty() {
            grim.arg("-g").arg(&self.geometry);
        }

        grim.arg(&self.output).execute()?;

        Ok(())
    }
}
