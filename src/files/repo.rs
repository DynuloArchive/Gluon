use serde::{Serialize, Deserialize};
use serde_json;

use std::fs::File;
use std::io::{Error, Read, Write};

use crate::error::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    pub l: Vec<Layer>,
    pub n: String,
    pub v: u16,
}
impl Repo {
    pub fn new(name: String) -> Self {
        Repo {
            l: Vec::new(),
            n: name,
            v: 1,
        }
    }
    pub fn save(&self) -> Result<(), Error> {
        let mut out = File::create("repo.cbor")?;
        serde_cbor::to_writer(&mut out, &self).unwrap_or_print();
        let mut out = File::create("repo.json")?;
        out.write_fmt(format_args!("{}", serde_json::to_string_pretty(&self)?))?;
        Ok(())
    }
    pub fn open<I: Read>(reader: &mut I) -> Result<Self, Error> {
        let repo: Repo = serde_cbor::from_reader(reader).unwrap_or_print();;
        Ok(repo)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Layer {
    pub n: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub f: Vec<ModFile>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub l: Vec<Layer>,
}
impl Layer {
    pub fn new(name: String) -> Self {
        Layer {
            n: name,
            f: Vec::new(),
            l: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModFile {
    pub n: String,
    pub h: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub p: Vec<ModPart>,
}
impl ModFile {
    pub fn new(name: String, hash: String) -> Self {
        ModFile {
            n: name,
            h: hash,
            p: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModPart {
    pub n: String,
    pub h: String,
    pub l: usize,
    pub s: usize,
}
impl ModPart {
    pub fn new(name: String, hash: String, size: usize, start: usize) -> Self {
        ModPart {
            n: name,
            h: hash,
            l: size,
            s: start,
        }
    }
}
