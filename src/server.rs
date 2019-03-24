use ws::{listen, CloseCode, Handler, Message, Result, Sender, Handshake};

use std::thread;
use std::path::PathBuf;

enum Stage {
    ROOT,
    DIR,
    FETCH,
}

pub fn run() {
    struct Server {
        out: Sender,
        stage: Stage,
        dir: Option<PathBuf>,
    }
    impl Handler for Server {
        fn on_open(&mut self, _: Handshake) -> Result<()> {
            self.out.send("1")
        }
        fn on_message(&mut self, msg: Message) -> Result<()> {
            println!("Server Got: {}", msg);
            let text = msg.into_text().unwrap();
            match &self.stage {
                Stage::ROOT => {
                    match text.as_str() {
                        "dir" => {
                            self.stage = Stage::DIR;
                            self.out.send("fetch").unwrap();
                        }
                        _ => {}
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
                                    crate::functions::fetch::process(dir.to_path_buf(), &url);
                                });
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        fn on_close(&mut self, _: CloseCode, _: &str) {
            println!("Lost Connection");
        }
    }

    listen("127.0.0.1:51462", |out| Server {out, stage: Stage::ROOT, dir: None}).unwrap();
}
