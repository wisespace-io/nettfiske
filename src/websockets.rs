use crate::errors::*;
use url::Url;
use tungstenite::connect;
use tungstenite::Message;
use tungstenite::protocol::WebSocket;
use tungstenite::client::AutoStream;
use tungstenite::handshake::client::Response;
use std::sync::mpsc::{self, channel};

static WEBSOCKET_URL: &'static str = "wss://certstream.calidog.io";

pub trait EventHandler {
    fn on_connect(&mut self);
    fn on_data_event(&mut self, event: String);
    fn on_error(&mut self, message: Error); 
}

#[allow(dead_code)]
#[derive(Debug)]
enum WsMessage {
    Close,
    Text(String),
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Sender {
    tx: mpsc::Sender<WsMessage>
}

pub struct WebSockets {
    socket: Option<(WebSocket<AutoStream>, Response)>,
    _sender: Sender,
    event_handler: Option<Box<dyn EventHandler>>, 
}

impl WebSockets {
    pub fn new() -> WebSockets {
        let (tx, _rx) = channel::<WsMessage>();
        let sender = Sender {
            tx: tx
        };

        WebSockets {
            socket: None,
            _sender: sender,
            event_handler: None
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        let url = Url::parse(&WEBSOCKET_URL.to_string())?;

        match connect(url) {
            Ok(answer) => {
                self.socket = Some(answer);
                if let Some(ref mut h) = self.event_handler {
                    h.on_connect();
                }                
                Ok(())
            }
            Err(e) => {
                bail!(format!("Error during handshake {}", e))
            }
        }
    }

    pub fn add_event_handler<H>(&mut self, handler: H) where H: EventHandler + 'static {
        self.event_handler = Some(Box::new(handler));
    }

    pub fn event_loop(&mut self) -> Result<()>  {
        loop {
            if let Some(ref mut socket) = self.socket {
                let message = socket.0.read_message()?;

                match message {
                    Message::Text(text) => {
                        if let Some(ref mut h) = self.event_handler {
                            h.on_data_event(text);
                        }
                    }
                    Message::Binary(_) => {}
                    Message::Ping(_) |
                    Message::Pong(_) => {}
                    Message::Close(_) => {}
                }
            }
        }
    } 
}
