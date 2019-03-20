use data_encoding::BASE64_NOPAD;
use ring::digest::{Context, Digest, SHA256};

use std::fs::File;
use std::io::{BufReader, Read, Error};
use std::io::Cursor;
use std::path::PathBuf;

pub fn hash_digest<R: Read>(mut reader: R) -> Result<Digest, Error> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

pub fn hash_file(path: &PathBuf) -> Result<String, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(BASE64_NOPAD.encode(hash_digest(reader)?.as_ref()))
}

pub fn hash_cursor(cursor: Cursor<Box<[u8]>>) -> Result<String, Error> {
    Ok(BASE64_NOPAD.encode(hash_digest(cursor)?.as_ref()))
}
