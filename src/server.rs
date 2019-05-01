use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use ws::{connect, listen, CloseCode, Handler, Message, Result, Sender, Handshake};

use std::path::PathBuf;
use std::thread;

use crate::error::*;

#[derive(Clone)]
pub enum Stage {
    ROOT,
    DIR,
    URL,
    CMD,
}

#[derive(Clone)]
pub struct Server {
    pub id: String,
    pub out: Sender,
    pub stage: Stage,
    pub dir: Option<PathBuf>,
    pub url: Option<String>,
}
impl Handler for Server {
        fn on_open(&mut self, _: Handshake) -> Result<()> {
            self.out.send("1")
        }
        fn on_message(&mut self, msg: Message) -> Result<()> {
            let text = msg.into_text().unwrap();
            match &self.stage {
                Stage::ROOT => {
                    match text.as_str() {
                        "dir" => {
                            self.stage = Stage::DIR;
                            self.out.send("dir").unwrap();    
                            println!("[{}] New Connection", self.id);
                        }
                        _ => {
                            self.out.broadcast(text);
                            self.out.close(CloseCode::Normal);
                        }
                    }
                },
                Stage::DIR => {
                    match text.as_str() {
                        _ => {
                            self.stage = Stage::URL;
                            self.dir = Some(PathBuf::from(&text));
                            println!("[{}] Directory: {}", self.id, &text);
                            self.out.send("url").unwrap();
                        }
                    }
                },
                Stage::URL => {
                    match text.as_str() {
                        _ => {
                            if !text.starts_with("http") {
                                self.out.send("missing http").unwrap();
                            } else {
                                println!("url: {}", text);
                                self.stage = Stage::CMD;
                                self.url = Some(text.clone());
                                println!("[{}] URL: {}", self.id, &text);
                                self.out.send("cmd").unwrap();
                            }
                        }
                    }
                },
                Stage::CMD => {
                    match text.as_str() {
                        "verify" => {
                            let url = self.url.clone().unwrap();
                            let dir = self.dir.clone().unwrap();
                            thread::spawn(move || {
                                crate::functions::fetch::verify(dir.to_path_buf(), url).unwrap_or_print();
                            });
                        },
                        "fetch" => {
                            let url = self.url.clone().unwrap();
                            let dir = self.dir.clone().unwrap();
                            thread::spawn(move || {
                                crate::functions::fetch::process(dir.to_path_buf(), url).unwrap_or_print();
                            });
                        },
                        _ => {
                            self.out.send("unknown")?;
                        }
                    }
                }
            }
            Ok(())
        }
    }

pub fn run() {
    listen("127.0.0.1:51462", |out| {
        let id: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .collect();
        Server {out, stage: Stage::ROOT, id: id, dir: None, url: None}
    }).unwrap();
}

// real hacky, a better solution would be nice
pub fn send(text: String) {
    thread::spawn(move || {
        connect("ws://127.0.0.1:51462", |out| {
            out.send(text.clone()).unwrap();
            move |_| {
                out.close(CloseCode::Normal)
            }
        })
    });
}
