use reqwest;
use serde_json;

use std::io::{Cursor, Error};

use crate::error::*;
use crate::files::packages::*;

const REPO: &'static str = "https://raw.githubusercontent.com/Dynulo/GluonRepository/master/";

pub fn add(package: &String, total: u32) -> Result<u32, Error> {
    let mut p: Packages = Packages::open()?;
    let url = format!("{}{}/{}.json", REPO, &package[0..2], package);
    let mut response = reqwest::get(&url).unwrap_or_print();
    let remote: RemotePackage = serde_json::from_reader(&mut response)?;
    println!("{}", remote.name);
    let github_url = format!("https://api.github.com/repos/{}/releases", remote.github);
    let mut response = reqwest::get(&github_url).unwrap_or_print();
    let releases: Vec<Release> = serde_json::from_reader(&mut response)?;
    let buf: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(buf);
    crate::download::download(&releases[0].assets[0].url, &mut cursor, Some(releases[0].assets[0].size)).unwrap_or_print();
    let folders = crate::files::extract(cursor)?;
    p.packages.insert(package.clone(), LocalPackage {
        github: remote.github,
        version: releases[0].tag_name.clone(),
        folders: folders,
    });
    let mut addtotal = 1;
    for depend in &remote.depends {
        addtotal += add(depend, total)?;
    }
    Ok(total + addtotal)
}
