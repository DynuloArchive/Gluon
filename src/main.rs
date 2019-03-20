use docopt::Docopt;
use serde::Deserialize;
use num_cpus;

#[cfg(windows)]
use ansi_term;

use std::io::{Error};

mod error;
mod files;
mod functions;
mod hash;

use crate::error::*;

const USAGE: &'static str = "
Gluon, an easy to use PBO management tool

Usage:
    gluon run <config> [--jobs=<n>]
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_run: bool,
    arg_config: String,
    flag_jobs: usize,
}

fn run(args: &Args) -> Result<(), Error> {
    if args.cmd_run {
        crate::functions::run::process(&args.arg_config)?;
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
        args.flag_jobs = num_cpus::get();
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
