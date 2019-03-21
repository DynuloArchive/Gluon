use armake2::pbo;
use rayon::prelude::*;

use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::error::*;
use crate::files::*;

pub fn process() -> Result<(), Error> {
    let c = config::open(&PathBuf::from("./config.toml"))?;
    println!("{:?}", c.mods);
    let mut repo = Repo::new(c.name.clone());
    let arepo = Arc::new(Mutex::new(&mut repo));
    c.mods.par_iter().for_each(|entry| {
        let mut layer = Layer::new(entry.1.path.clone());
        let alayer = Arc::new(Mutex::new(&mut layer));
        process_dir(&entry.1.path, &alayer).unwrap_or_print();
        arepo.lock().unwrap().l.push(layer);
    });
    repo.save()?;
    c.save()?;
    Ok(())
}

pub fn process_dir(dir: &String, layer: &Arc<Mutex<&mut Layer>>) -> Result<(), Error> {
    let files: Vec<PathBuf> = fs::read_dir(dir).unwrap()
        .map(|file| file.unwrap().path())
        .collect();
    files.par_iter().for_each(|entry| {
        if entry.is_file() {
            let mut file = ModFile::new(
                entry.file_name().unwrap().to_str().unwrap().to_owned(),
                crate::hash::hash_file(entry).unwrap_or_print()
            );
            if entry.as_path().extension().unwrap_or(OsStr::new("")) == "pbo" {
                let pbofile = pbo::PBO::read(&mut File::open(entry).unwrap_or_print()).unwrap_or_print();
                // add 2, null start byte, null end byte
                // add 21, empty PBO header
                let mut headersize: usize = (std::mem::size_of::<u32>() * 5) + 23;
                for header in pbofile.headers {
                    //println!("{:?} {:?}", header.filename, header.packing_method);
                    let size = (std::mem::size_of::<u32>() * 5) + header.filename.len() + 1;
                    headersize += size;
                }
                for header in pbofile.header_extensions {
                    // add 2, for null terminators
                    let size = header.0.len() + header.1.len() + 2;
                    headersize += size;
                }
                let mut filestart: usize = headersize;
                for filepart in &pbofile.files {
                    let hash = crate::hash::hash_cursor(filepart.1.clone()).unwrap_or_print();
                    let length = filepart.1.get_ref().clone().len();
                    let part = ModPart::new(filepart.0.to_string(), hash, length.clone(), filestart);
                    filestart += length;
                    file.p.push(part);
                }
            }
            layer.lock().unwrap().f.push(file);
        } else {
            let mut newlayer = Layer::new(entry.file_name().unwrap().to_str().unwrap().to_owned());
            let alayer = Arc::new(Mutex::new(&mut newlayer));
            process_dir(&(entry.clone()).into_os_string().to_str().unwrap().to_owned(), &alayer).unwrap_or_print();
            layer.lock().unwrap().l.push(newlayer);
        }
    });
    Ok(())
}
