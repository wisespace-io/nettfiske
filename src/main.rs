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

mod util;
mod data;

use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use util::CertString;
use serde_json::{from_str};
use console::{Emoji, style};
use publicsuffix::List;
use clap::{Arg, App};
use ws::{connect, CloseCode, Error, ErrorKind, Handler, Handshake, Message, Result as WS_RESULT, Sender};

static LOOKING_GLASS: Emoji = Emoji("🔍  ", "");
static WEBSOCKET_URL: &'static str = "wss://certstream.calidog.io";

enum Event {
    Connect(Sender),
    Disconnect,
}

struct Client {
    list: List,
    config: data::Config,
    ws_out: Sender,
    thread_out: TSender<Event>,
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> WS_RESULT<()> {
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
                    for mut domain in cert.data.leaf_cert.all_domains {
                        if domain.starts_with("*.") {
                            domain = domain.replace("*.", "");
                        }
                        
                        util::analyse_domain(&domain, &mut self.list, self.config.clone());
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
            util::display(format!("<<< Connection Closed by CertStream: <({:?})>", code));
        } else {
            util::display(format!("<<< Connection Closed by CertStream: <({:?}) {}>", code, reason));
        }

        if let Err(err) = self.thread_out.send(Event::Disconnect) {
            util::display(format!("{:?}", err))
        }
    }

    fn on_error(&mut self, err: Error) {
        util::display(format!("<<< Error<{:?}>", err))
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
        let json_config = util::open_json_config(file_name).unwrap();

        let config: data::Config = serde_json::from_str(&json_config).unwrap();

        match util::setup_logger() {
            Err(why) => panic!("{}", why),
            Ok(_) => (),
        };

        let url: String = format!("{}", WEBSOCKET_URL);
        let (tx, rx) = channel();

        let client = thread::spawn(move || {
            connect(url, |sender| Client {
                list: List::fetch().unwrap(),
                config: config.clone(),
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