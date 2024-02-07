use std::{net::{TcpListener, TcpStream}, thread::spawn};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response}, WebSocket,
};
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct WsRequest {
    c: i32,
}

///
/// Gets the current unix timestamp of the server
/// Returns - i64 - The current unix timestamp
///
fn get_timestamp() -> i64{

    //Gets the current unix timestamp of the server
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => return n.as_secs() as i64,
        Err(_e) =>return 0
    }

}

fn get_event(c: i32) -> String {

    //create an event array for the time that message "c" is received by the server
    let event = json!({
        "c": c,
        "ts": get_timestamp()
    });

    // convert the json to a string
    let event_string = event.to_string();

    event_string
}

fn notify(mut websocket: WebSocket<TcpStream>, c: i32 ) -> WebSocket<TcpStream> {
    let message = get_event(c);

    //send the given connection the event timestamp for message "c"
    websocket.send(tungstenite::protocol::Message::Text(message)).unwrap();

    websocket
}

fn main() {
    let server = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |_: &Request, response: Response| { Ok(response) };
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            websocket = notify(websocket, 0);

            loop {
                let msg = websocket.read().unwrap();
                if msg.is_binary() || msg.is_text() {
                    
                    match msg {
                        // process any incoming Text messages
                        tungstenite::protocol::Message::Text(message_text) => {
    
                            // decode incoming message into a struct
                            let request: WsRequest = serde_json::from_str(&message_text).unwrap();
    
                            // notify client with event for message with count "c"
                            websocket = notify(websocket, request.c)
                        }
                        //ignore all other kinds of message ( Ping, Close, etc )
                        _ => {
                            continue;
                        }
                    }
                }
            }
        });
    }
}
