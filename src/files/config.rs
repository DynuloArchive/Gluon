use serde::{Serialize, Deserialize};
use toml;

use std::collections::{HashMap};
use std::fs::File;
use std::io::{Error, Write};
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
    pub name: String,
    pub mods: HashMap<String, Mod>,
    pub serers: HashMap<String, Server>,
}
impl Config {
    pub fn save(&self) -> Result<(), Error> {
        let mut out = File::create("config.cbor")?;
        serde_cbor::to_writer(&mut out, &self).unwrap_or_print();
        let mut out = File::create("config.json")?;
        out.write_fmt(format_args!("{}", serde_json::to_string_pretty(&self)?))?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mod {
    pub path: String,
    #[serde(default = "dft_false")]
    pub optional: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    pub address: String,
    pub server: String,
    pub name: String,
    #[serde(default = "dft_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub primary: bool,
}

fn is_false(v: &bool) -> bool { !v.clone() }
fn dft_false() -> bool { false }
