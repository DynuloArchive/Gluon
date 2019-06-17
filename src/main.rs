use docopt::Docopt;
use num_cpus;
use serde::Deserialize;

#[cfg(windows)]
use ansi_term;

use std::io::{Error};
use std::path::PathBuf;

mod download;
mod error;
mod files;
mod functions;
mod hash;
mod server;

use crate::error::*;
use crate::files::packages::*;

const USAGE: &str = "
Gluon, an easy to use mod distribution tool

Usage:
    gluon run [--jobs=<n>]
    gluon fetch <dir> <config>
    gluon add <package>
    gluon update
    gluon server
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_run: bool,
    cmd_fetch: bool,
    cmd_add: bool,
    cmd_update: bool,
    cmd_server: bool,
    arg_dir: String,
    arg_config: String,
    arg_package: String,
    flag_jobs: usize,
}

fn run(args: &Args) -> Result<(), Error> {
    if args.cmd_run {
        crate::functions::run::process()?;
    } else if args.cmd_fetch {
        crate::functions::fetch::process(PathBuf::from(&args.arg_dir), args.arg_config.clone())?;
    } else if args.cmd_add {
        let mut p: Packages = Packages::open()?;
        let installed = crate::functions::repo::add(&mut p, &args.arg_package, 0)?;
        p.save()?;
        println!("Installed {} Packages", installed);
    } else if args.cmd_update {
        let mut p: Packages = Packages::open()?;
        let installed = crate::functions::update::process(&mut p)?;
        p.save()?;
        println!("Updated {} Packages", installed);
    } else if args.cmd_server {
        println!("Starting Gluon Server");
        crate::server::run();
    }
    Ok(())
}

fn main() {
    if cfg!(windows) {
        ansi_support();
    }

    let mut args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_jobs == 0 {
        args.flag_jobs = std::cmp::min(4, num_cpus::get());
    }
    rayon::ThreadPoolBuilder::new().num_threads(args.flag_jobs).build_global().unwrap();

    run(&args).unwrap_or_print();
}

#[cfg(windows)]
fn ansi_support() {
    // Attempt to enable ANSI support in terminal
    // Disable colored output if failed
    if !ansi_term::enable_ansi_support().is_ok() {
        colored::control::set_override(false);
    }
}

#[cfg(not(windows))]
fn ansi_support() {
    unreachable!();
}
