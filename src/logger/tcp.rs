use std::{io::Write, net::TcpListener, sync::mpsc::{self, Sender}, thread};

use log::{Level, Metadata, Record};
use skyline::{libc::memalign, nn};

pub struct TcpLogger(Sender<String>);

impl TcpLogger {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<String>();

        std::thread::Builder::new()
            .name("tcp-logger".into())
            .spawn(move || {
                let pool = unsafe { memalign(0x1000, 0x100000) as *mut u8 };
                
                unsafe { nn::socket::Initialize(pool, 0x100000, 0x20000, 14) };

                let listener = TcpListener::bind("0.0.0.0:6969").unwrap();
                
                if let Some(Ok(mut stream)) = listener.incoming().next() {
                    thread::spawn(move || {
                        loop {
                            match receiver.recv() {
                                Ok(message) => { 
                                    let _ = stream.write_all(message.as_bytes());
                                },
                                Err(err) => {
                                    println!("Listener thread ran into an error: {}", err);
                                },
                            }
                        }
                    });
                }
        }).unwrap();

        TcpLogger(sender)
    }
}

impl log::Log for TcpLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let _ = self.0.send(record.args().to_string());
        }
    }

    fn flush(&self) {}
}