use std::convert::TryInto;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use websocket::server::InvalidConnection;
use websocket::sync::{Server, Client};
use hyper::http::h1::Incoming;
use hyper::method::Method;
use hyper::uri::RequestUri;
use websocket::{OwnedMessage, CloseData};
use utf8::BufReadDecoder;

mod child;
use child::Child;

fn main() {
    let server = Server::bind("0.0.0.0:1234").unwrap();

    println!("Running");

    for req in server {
        match req {
            Ok(req) => {
                thread::spawn(move || {
                    handle_websocket(req.accept().unwrap());
                });
            },
            Err(InvalidConnection {
                stream: Some(stream),
                parsed: Some(req),
                ..
            }) => {
                match req {
                    Incoming {
                        subject: (Method::Get, RequestUri::AbsolutePath(ref path)),
                        ..
                    }
                        if path == "/" || path.starts_with("/?")
                    => {
                        //respond(stream, "200 OK", include_str!("../target/index.html"));
                        respond(
                            stream,
                            "200 OK",
                            &std::fs::read_to_string("target/index.html").unwrap());
                    },
                    _ => respond(stream, "404 Not Found", "Not Found"),
                }
            },
            _ => { },
        }
    }
}

fn respond(mut stream: TcpStream, status: &str, content: &str) {
    write!(stream, "HTTP/1.1 {}\r\nContent-type: text/html\r\n\r\n{}", status, content).ok();
}

fn handle_websocket(client: Client<TcpStream>) {
    let (mut receiver, sender) = client.split().unwrap();

    let sender = Arc::new(Mutex::new(sender));

    let mut child: Option<Child> = None;

    for msg in receiver.incoming_messages() {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => break,
        };

        match msg {
            OwnedMessage::Binary(msg) => {
                if msg.len() == 5 && msg[0] == 1 {
                    let rows = u16::from_be_bytes(msg[1..3].try_into().unwrap());
                    let cols = u16::from_be_bytes(msg[3..5].try_into().unwrap());

                    match child {
                        Some(ref mut child) => {
                            child.set_window_size(rows, cols);
                        },
                        None => {
                            let mut c = Child::spawn("/usr/bin/tmux", vec!["attach"], rows, cols)
                                .unwrap();
                            let child_reader = c.reader();
                            let child_sender = Arc::clone(&sender);

                            child = Some(c);

                            thread::spawn(move || {
                                let mut reader = BufReadDecoder::new(
                                    BufReader::with_capacity(1024, child_reader));

                                while let Some(Ok(s)) = reader.next_lossy() {
                                    if child_sender.lock().unwrap()
                                        .send_message(&OwnedMessage::Text(s.to_owned()))
                                        .is_err()
                                    {
                                        break;
                                    }
                                }

                                child_sender.lock().unwrap()
                                    .send_message(&OwnedMessage::Close(Some(CloseData {
                                        status_code: 1,
                                        reason: String::from(""),
                                    })))
                                    .ok();
                            });
                        }
                    }
                }
            }
            OwnedMessage::Close(_) => {
                break;
            },
            OwnedMessage::Ping(ping) => {
                sender.lock().unwrap()
                    .send_message(&OwnedMessage::Pong(ping)).ok();
            },
            _ => { },
        }
    }
}
