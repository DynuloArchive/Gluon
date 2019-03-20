use reqwest;

use std::io::{Error};
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use crate::error::*;
use crate::files::*;

pub fn process(config: &String) -> Result<(), Error> {
    let mut body = reqwest::get(&format!("{}/repo.cbor", &config)).unwrap_or_print();
    let repo = crate::files::Repo::open(&mut body)?;
    for moddir in &repo.l {
        println!("Mod: {}", moddir.n);
        process_layer(&config, &String::from("."), moddir)?;
    }
    Ok(())
}

pub fn process_layer(root: &String, lpath: &String, layer: &Layer) -> Result<(), Error> {
    let path = format!("{}/{}", &lpath, &layer.n);
    println!("layer: {}", path);
    for e in fs::read_dir(format!("mods/{}", &path))? {
        let epath = e?.path();
        let name = epath.file_name().unwrap().to_str().unwrap().to_owned();
        if epath.is_dir() {
            if layer.l.iter().filter(|&x| *x.n == name).collect::<Vec<_>>().len() == 0 {
                fs::remove_dir_all(format!("mods/{}/{}", &path, name))?;
            }
        } else {
            if layer.f.iter().filter(|&x| *x.n == name).collect::<Vec<_>>().len() == 0 {
                fs::remove_file(format!("mods/{}/{}", &path, name))?;
            }
        }
    }

    fs::create_dir_all(format!("mods/{}", &path))?;
    for slayer in &layer.l {
        process_layer(&root, &path, slayer)?;
    }
    for file in &layer.f {
        let pbuf = PathBuf::from(&format!("mods/{}/{}", &path, &file.n));
        let mut download = false;
        if pbuf.exists() {
            if crate::hash::hash_file(&pbuf)? != file.h {
                if pbuf.extension().unwrap() == "pbo" {
                    println!("pbo mismatch");
                } else {
                    download = true;
                }
            }
        } else {
            download = true;
        }
        if download {
            let mut urlpath = path.clone();
            urlpath.remove(0);
            urlpath.remove(0);
            let url = format!("{}/{}/{}", &root, &urlpath, &file.n);
            println!("file: {}", &url);
            let mut response = reqwest::get(&url).unwrap_or_print();
            let mut out = File::create(pbuf)?;
            std::io::copy(&mut response, &mut out)?;
        }
    }
    Ok(())
}
