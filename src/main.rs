#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate strsim;
extern crate console;
extern crate publicsuffix;
extern crate idna;
extern crate unicode_skeleton;
extern crate ws;
extern crate clap;

mod nettfiske;
mod data;

use nettfiske::{Nettfiske};
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use data::CertString;
use serde_json::{from_str};
use console::{Emoji, style};
use clap::{Arg, App};
use std::fs::File;
use std::io::{BufReader, Error as IOError, prelude::*};
use ws::{connect, CloseCode, Error, ErrorKind, Handler, Handshake, Message, Result as WS_RESULT, Sender};

static LOOKING_GLASS: Emoji = Emoji("🔍  ", "");
static WEBSOCKET_URL: &'static str = "wss://certstream.calidog.io";

enum Event {
    Connect(Sender),
    Disconnect,
}

struct Client {
    nettfiske: Nettfiske,
    ws_out: Sender,
    thread_out: TSender<Event>,
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> WS_RESULT<()> {
        match self.nettfiske.setup_logger() {
            Err(why) => panic!("{}", why),
            Ok(_) => (),
        };

        self.thread_out
            .send(Event::Connect(self.ws_out.clone()))
            .map_err(|err| {
                Error::new(
                    ErrorKind::Internal,
                    format!("Unable to connect: {:?}.", err),
                )
            })
    }

    fn on_message(&mut self, msg: Message) -> WS_RESULT<()> {
        let msg_txt = msg.as_text()?;

        match from_str(msg_txt) {
            Ok(message) => {
                let cert: CertString = message;
                if cert.message_type.contains("certificate_update") {
                    for domain in cert.data.leaf_cert.all_domains {
                        self.nettfiske.analyse_domain(&domain, cert.data.chain.clone());
                    }
                }
            } Err(e) => {
                error!("Received unknown message: {}", e);
                return Ok(());
            }
        }

        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        if reason.is_empty() {
            display(format!("<<< Connection Closed by CertStream: <({:?})>", code));
        } else {
            display(format!("<<< Connection Closed by CertStream: <({:?}) {}>", code, reason));
        }

        if let Err(err) = self.thread_out.send(Event::Disconnect) {
            display(format!("{:?}", err))
        }
    }

    fn on_error(&mut self, err: Error) {
        display(format!("<<< Error<{:?}>", err))
    }
}

fn main() {
    let matches = App::new("Nettfiske")
                        .args(&[
                            Arg::with_name("input")
                                    .help("the input file to use")
                                    .index(1)
                                    .required(true)
                        ]).get_matches();

    if let Some(file_name) = matches.value_of("input") {
        let json_config = open_json_config(file_name).unwrap();
        let config: data::Config = serde_json::from_str(&json_config).unwrap();
        let url: String = format!("{}", WEBSOCKET_URL);
        let (tx, rx) = channel();

        let client = thread::spawn(move || {
            connect(url, |sender| Client {
                nettfiske: Nettfiske::new(config.clone()),
                ws_out: sender,
                thread_out: tx.clone(),
            }).unwrap();
        });

        if let Ok(Event::Connect(_sender)) = rx.recv() {
            println!("{} {} Fetching Certificates ...", style("[Nettfiske]").bold().dim(), LOOKING_GLASS);
        }

        // Ensure the client has a chance to finish up
        client.join().unwrap();        
    }
}

fn display(string: String) {
    println!("{}", string);
}

fn open_json_config(file_name: &str) -> Result<String, IOError> {
    let file = File::open(file_name)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
 
    Ok(contents)    
}