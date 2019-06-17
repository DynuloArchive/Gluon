use http::header::HeaderValue;
use reqwest;
use reqwest::Client;
use reqwest::header::{IF_NONE_MATCH};
use serde_json;

use std::io::{Cursor, Error};

use crate::error::*;
use crate::files::packages::*;

const REPO: &str = "https://raw.githubusercontent.com/Dynulo/GluonRepository/master/";

pub fn process(p: &mut Packages) -> Result<u32, Error> {

    let mut total = 0;

    let old = p.packages.clone();

    for (package, _v) in old {
        let url = format!("{}{}/{}.json", REPO, &package[0..2], package);
        let mut response = reqwest::get(&url).unwrap_or_print();
        let remote: RemotePackage = serde_json::from_reader(&mut response)?;
        let github_url = format!("https://api.github.com/repos/{}/releases", remote.github);
        let client = Client::new();
        let request = if p.packages.contains_key(&package) {
            client.get(&github_url).header(IF_NONE_MATCH, p.packages[&package].etag.clone())
        } else {
            client.get(&github_url)
        };
        let mut response = request.send().unwrap_or_print();
        if response.status().as_u16() != 304 {
            let releases: Vec<Release> = serde_json::from_reader(&mut response)?;
            if p.packages.contains_key(&package) {
                p.packages.remove(&package);
            }
            println!("{}", remote.name);
            let buf: Vec<u8> = Vec::new();
            let mut cursor = Cursor::new(buf);
            crate::download::download(&releases[0].assets[0].url, &mut cursor, Some(releases[0].assets[0].size), None).unwrap_or_print();
            let folders = crate::files::extract(cursor)?;
            total += 1;
            if p.packages.contains_key(&package) {
                p.packages.remove(&package);
            }
            p.packages.insert(package.clone(), LocalPackage {
                github: remote.github,
                version: releases[0].tag_name.clone(),
                folders,
                etag: response.headers().get("ETAG").unwrap_or(&HeaderValue::from_str("").unwrap()).to_str().unwrap().to_owned(),
            });
        }
    }

    Ok(total)
}
