pub mod config;
pub mod packages;
pub mod repo;

use zip;

pub use crate::files::repo::*;

use std::fs;
use std::io::{Error, Read, Seek};

use crate::error::*;

pub fn extract<R: Seek + Read>(input: R) -> Result<Vec<String>, Error> {
    let mut archive = zip::ZipArchive::new(input)?;
    let mut roots: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap_or_print();
        let outpath = file.sanitized_name();
        if (&*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
            let path = outpath.as_path().components().next().unwrap().as_os_str().to_str().unwrap().to_owned();
            let is_present = roots.iter().any(|c| c == &path);
            if !is_present {
                roots.push(path);
            }
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p.to_str().unwrap().to_owned().to_lowercase())?;
                }
            }
            let mut outfile = fs::File::create(&outpath.as_path().to_str().unwrap().to_owned().to_lowercase())?;
            std::io::copy(&mut file, &mut outfile)?;
        }
        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(roots)
}
