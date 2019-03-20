use serde::{Serialize, Deserialize};
use toml;

use std::collections::{HashMap};
use std::fs::File;
use std::io::{Error};
use std::io::Read;
use std::path::PathBuf;

use crate::error::*;

pub fn open(path: &PathBuf) -> Result<Config, Error> {
    let mut f = File::open(path)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    let c: Config = toml::from_str(contents.as_str()).unwrap_or_print();
    Ok(c)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub mods: HashMap<String, Mod>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mod {
    pub path: String,
    #[serde(default = "def_false")]
    pub optional: bool,
}

pub fn def_false() -> bool {
    false
}
