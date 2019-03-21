use serde::{Serialize, Deserialize};
use toml;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::Path;

use crate::error::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Packages {
    pub packages: HashMap<String, LocalPackage>,
}
impl Packages {
    pub fn open() -> Result<Self, Error> {
        let p: Packages = if Path::new("packages.toml").exists() {
            let mut f = File::open("packages.toml")?;
            let mut contents = String::new();
            f.read_to_string(&mut contents)?;
            toml::from_str(contents.as_str()).unwrap_or_print()
        } else {
            Packages {
                packages: HashMap::new()
            }
        };
        Ok(p)
    }
    pub fn save(&self) -> Result<(), Error> {
        let mut f = File::create("packages.toml")?;
        f.write_fmt(format_args!("{}", &toml::to_string_pretty(&self).unwrap_or_print()))?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalPackage {
    pub version: String,
    pub github: String,
    pub folders: Vec<String>,
    #[serde(default = "String::new")]
    pub etag: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemotePackage {
    pub name: String,
    pub author: String,
    pub github: String,
    #[serde(default = "Vec::new")]
    pub depends: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
    pub draft: bool,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReleaseAsset {
    pub url: String,
    pub size: u64,
    pub content_type: String,
}
