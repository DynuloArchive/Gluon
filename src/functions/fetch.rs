use armake2::pbo::{PBO, PBOHeader};
use armake2::io::*;
use reqwest;
use reqwest::header::{HeaderValue, RANGE};
use linked_hash_map::{LinkedHashMap};

use std::collections::HashMap;
use std::io::{Cursor, Error, Read};
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
    if PathBuf::from(format!("mods/{}", &path)).exists() {
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
    }

    fs::create_dir_all(format!("mods/{}", &path))?;
    for slayer in &layer.l {
        process_layer(&root, &path, slayer)?;
    }
    for file in &layer.f {
        let pbuf = PathBuf::from(&format!("mods/{}/{}", &path, &file.n));
        let mut download = false;
        let mut urlpath = path.clone();
        urlpath.remove(0);
        urlpath.remove(0);
        let url = format!("{}/{}/{}", &root, &urlpath, &file.n);
        if pbuf.exists() {
            if crate::hash::hash_file(&pbuf)? != file.h {
                if pbuf.extension().unwrap() == "pbo" {
                    // https://www.youtube.com/watch?v=KAHLwAxS7FI
                    let pbofile = PBO::read(&mut File::open(&pbuf).unwrap_or_print()).unwrap_or_print();
                    let client = reqwest::Client::new();
                    let mut response = client.get(&url)
                        .header(RANGE,
                            HeaderValue::from_str(&format!("bytes={}-{}", 0, file.p[0].s - 1)).unwrap_or_print()
                        ).send().unwrap_or_print();
                    let mut headers: Vec<PBOHeader> = Vec::new();
                    let mut first = true;
                    let mut header_extensions: HashMap<String, String> = HashMap::new();
                    loop {
                        let header = PBOHeader::read(&mut response)?;
                        if header.packing_method == 0x56657273 {
                            if !first { unreachable!(); }
                            loop {
                                let s = response.read_cstring()?;
                                if s.len() == 0 { break; }
                                header_extensions.insert(s, response.read_cstring()?);
                            }
                        } else if header.filename == "" {
                            break;
                        } else {
                            headers.push(header);
                        }
                        first = false;
                    }
                    let mut files: LinkedHashMap<String, Cursor<Box<[u8]>>> = LinkedHashMap::new();
                    for header in &headers {
                        let newfile = file.p.iter().filter(|&x| x.n == header.filename).collect::<Vec<_>>()[0];
                        if pbofile.headers.iter().filter(|&x| *x.filename == header.filename).collect::<Vec<_>>().len() == 0 {
                            println!("new file: {}", header.filename);
                            let client = reqwest::Client::new();
                            let mut response = client.get(&url)
                                .header(RANGE,
                                    HeaderValue::from_str(&format!("bytes={}-{}", newfile.s, newfile.s + newfile.l)).unwrap_or_print()
                                ).send().unwrap_or_print();
                            let mut buffer: Box<[u8]> = vec![0; newfile.l as usize].into_boxed_slice();
                            response.read_exact(&mut buffer)?;
                            files.insert(header.filename.clone(), Cursor::new(buffer));
                        } else {
                            if crate::hash::hash_cursor(pbofile.files.get(&header.filename).unwrap().clone())? != newfile.h {
                                println!("hash part mismatch: {}", header.filename);
                                let client = reqwest::Client::new();
                                let mut response = client.get(&url)
                                    .header(RANGE,
                                        HeaderValue::from_str(&format!("bytes={}-{}", newfile.s, newfile.s + newfile.l)).unwrap_or_print()
                                    ).send().unwrap_or_print();
                                let mut buffer: Box<[u8]> = vec![0; newfile.l as usize].into_boxed_slice();
                                response.read_exact(&mut buffer)?;
                                files.insert(header.filename.clone(), Cursor::new(buffer));
                            } else {
                                files.insert(header.filename.clone(), pbofile.files.get(&header.filename).unwrap().clone());
                            }
                        }
                    }
                    let newpbo = PBO {
                        files: files,
                        header_extensions: header_extensions,
                        headers: headers,
                        checksum: None,
                    };
                    let mut outfile = File::create(&pbuf)?;
                    newpbo.write(&mut outfile)?;
                } else {
                    download = true;
                }
            }
        } else {
            download = true;
        }
        if download {
            println!("file: {}", &url);
            let mut response = reqwest::get(&url).unwrap_or_print();
            let mut out = File::create(pbuf)?;
            std::io::copy(&mut response, &mut out)?;
        }
    }
    Ok(())
}
