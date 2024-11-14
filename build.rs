#[allow(dead_code)]
#[path = "src/cli/mod.rs"]
mod cli;

use clap::{Command, CommandFactory};
use clap_mangen::Man;
use std::{fs, path::PathBuf};

fn generate_man_pages(cmd: &Command) -> Result<(), Box<dyn std::error::Error>> {
    let man_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/man");
    fs::create_dir_all(&man_dir)?;

    let mut buffer = Vec::default();
    Man::new(cmd.clone()).render(&mut buffer)?;
    fs::write(man_dir.join("focal.1"), buffer)?;

    for subcmd in cmd.get_subcommands().filter(|c| !c.is_hide_set()) {
        let subcmd_name = format!("focal-{}", subcmd.get_name());
        let subcmd = subcmd.clone().name(&subcmd_name);

        let mut buffer = Vec::default();

        Man::new(subcmd)
            .title(subcmd_name.to_uppercase())
            .render(&mut buffer)?;

        fs::write(man_dir.join(subcmd_name + ".1"), buffer)?;
    }

    Ok(())
}

fn main() {
    // override with the version passed in from nix
    // https://github.com/rust-lang/cargo/issues/6583#issuecomment-1259871885
    if let Ok(val) = std::env::var("NIX_RELEASE_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={val}");
    }
    println!("cargo:rerun-if-env-changed=NIX_RELEASE_VERSION");

    let cmd = cli::Cli::command();
    if let Err(err) = generate_man_pages(&cmd) {
        println!("cargo::warning=Error generating man pages: {err}");
    }
}
