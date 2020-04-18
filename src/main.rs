#[macro_use] extern crate log;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate strsim;
extern crate console;
extern crate publicsuffix;
extern crate idna;
extern crate unicode_skeleton;
extern crate tungstenite;
extern crate clap;

mod nettfiske;
mod data;
mod errors;
mod websockets;

use crate::errors::*;
use crate::websockets::*;
use std::{thread, time};

use crate::nettfiske::{Nettfiske};
use crate::data::CertString;
use serde_json::{from_str};
use console::{Emoji, style};
use clap::{Arg, App};
use std::fs::File;
use std::io::{BufReader, prelude::*};

static LOOKING_GLASS: Emoji = Emoji("🔍  ", "");

struct WebSocketHandler {
    nettfiske: Nettfiske,
    logging: bool,
}

impl EventHandler for WebSocketHandler {
    fn on_connect(&mut self) {
        match self.nettfiske.setup_logger(self.logging) {
            Err(why) => error!("Error setting UP log: {}", why),
            Ok(_) => (),
        };
    }  

    fn on_data_event(&mut self, event: String) {
        match from_str(&event) {
            Ok(message) => {
                let cert: CertString = message;
                if cert.message_type.contains("certificate_update") {
                    for domain in cert.data.leaf_cert.all_domains {
                        self.nettfiske.analyse_domain(&domain, cert.data.chain.clone());
                    }
                }
            } Err(e) => {
                error!("Received unknown message: {}", e);
            }
        }
    }

    fn on_error(&mut self, message: Error) {
        display(format!("<<< Error<{:?}>", message));
    }    
}

fn main() {
    let matches = App::new("Nettfiske")
                        .args(&[
                            Arg::with_name("input")
                                    .help("the input file to use")
                                    .index(1)
                                    .required(true),
                            Arg::with_name("quiet")
                                    .help("Be less verbose")
                                    .short("q")
                                    .long("quiet"),
                            Arg::with_name("nolog")
                                    .help("Don't output log file")
                                    .long("nolog")
                        ]).get_matches();

    if let Some(file_name) = matches.value_of("input") {
        let json_config = open_json_config(file_name).unwrap();
        let config: data::Config = serde_json::from_str(&json_config).unwrap();

        let mut logging_enabled = !matches.is_present("nolog");
        let is_present = !matches.is_present("quiet");

        loop {
            if run(config.clone(), logging_enabled, is_present) == true {
                break;
            }
            logging_enabled = false; // log already initialized
        }   
    }
}

fn run(config: data::Config, logging_enabled: bool, is_present: bool) -> bool {
    let waiting_time = time::Duration::from_millis(5000);

    let mut web_socket: WebSockets = WebSockets::new();

    web_socket.add_event_handler(WebSocketHandler {
        nettfiske: Nettfiske::new(config.clone()),
        logging: logging_enabled,      
    });

    if let Ok(_answer) = web_socket.connect() {
        if is_present {
            println!("{} {} Fetching Certificates ...", style("[Nettfiske]").bold().dim(), LOOKING_GLASS);
        }
    }

    if let Err(_error) = web_socket.event_loop() {
        thread::sleep(waiting_time); // sleep 5s
        return false;
    }

    return true;
}

fn display(string: String) {
    println!("{}", string);
}

fn open_json_config(file_name: &str) -> Result<String> {
    let file = File::open(file_name)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    Ok(contents)
}
