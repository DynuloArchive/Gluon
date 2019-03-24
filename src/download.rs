use reqwest::Url;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use reqwest::header::{ACCEPT, CONTENT_LENGTH, USER_AGENT};

use std::io::{self, copy, Read, Write, Error};

use crate::error::*;

struct DownloadProgress<R> {
    inner: R,
    progress_bar: ProgressBar,
}

impl<R: Read> Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

pub fn download<O: Write>(url: &str, mut output: &mut O, size: Option<u64>, opb: Option<ProgressBar>) -> Result<ProgressBar, Error> {
    let url = Url::parse(url).unwrap_or_print();
    let client = Client::new();

    let total_size = match size {
        Some(v) => v,
        None => {
            let resp = client.head(url.as_str()).send().unwrap_or_print();
            if resp.status().is_success() {
                resp.headers()
                    .get(CONTENT_LENGTH)
                    .and_then(|ct_len| ct_len.to_str().ok())
                    .and_then(|ct_len| ct_len.parse().ok())
                    .unwrap_or(0)
            } else {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", resp.status())));
            }
        }
    };

    let request = client.get(url.as_str()).header(USER_AGENT, "Dynulo/Gluon").header(ACCEPT, "application/octet-stream");
    let pb = match opb {
        Some(v) => {
            v.set_length(total_size);
            v
        },
        None => ProgressBar::new(total_size)
    };
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-"));

    let mut source = DownloadProgress {
        progress_bar: pb,
        inner: request.send().unwrap_or_print(),
    };

    //output.write_all(&mut source.inner);
    let _ = copy(&mut source, &mut output)?;

    Ok(source.progress_bar)
}
