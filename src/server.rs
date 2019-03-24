use ws::{connect, listen, CloseCode, Handler, Message, Result, Sender, Handshake};

use std::path::PathBuf;
use std::thread;

use crate::error::*;

#[derive(Clone)]
pub enum Stage {
    ROOT,
    DIR,
    FETCH,
}

#[derive(Clone)]
pub struct Server {
    pub out: Sender,
    pub stage: Stage,
    pub dir: Option<PathBuf>,
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
                            self.out.send("send dir").unwrap();
                        }
                        _ => {
                            self.out.broadcast(text).unwrap();
                        }
                    }
                },
                Stage::DIR => {
                    match text.as_str() {
                        _ => {
                            self.stage = Stage::FETCH;
                            self.dir = Some(PathBuf::from(&text));
                            self.out.send("url").unwrap();
                        }
                    }
                },
                Stage::FETCH => {
                    match text.as_str() {
                        _ => {
                            if !text.starts_with("http") {
                                self.out.send("missing http").unwrap();
                            } else {
                                println!("url: {}", text);
                                let url = text.clone();
                                let dir = self.dir.clone().unwrap();
                                thread::spawn(move || {
                                    crate::functions::fetch::process(dir.to_path_buf(), url).unwrap_or_print();
                                });
                            }
                        }
                    }
                }
            }
            Ok(())
        }
    }

pub fn run() {
    listen("127.0.0.1:51462", |out| Server {out, stage: Stage::ROOT, dir: None}).unwrap();
}

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
