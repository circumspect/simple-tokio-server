use simple_server::api::{ClientMessage, ServerMessage};

use std::net::SocketAddr;
use futures::prelude::*;
use serde_json::Value;
use tokio::{net::{TcpListener, TcpStream}, sync::broadcast};
use tokio_serde::formats::*;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};


#[tokio::main]
pub async fn main() {
    // Bind a server socket
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    let (tx, _rx) = broadcast::channel::<(Value, SocketAddr)>(10);
    let mut user_count: usize = 0;

    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        user_count += 1;
        let user_number = user_count;
        println!("New user joined! User count: {}", user_count);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // Duplicate the socket: one for serializing and one for deserializing
        let socket = socket.into_std().unwrap();
        let socket2 = socket.try_clone().unwrap();
        let socket = TcpStream::from_std(socket).unwrap();
        let socket2 = TcpStream::from_std(socket2).unwrap();

        let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());
        let mut deserialized = tokio_serde::SymmetricallyFramed::new(
            length_delimited,
            SymmetricalJson::<ClientMessage>::default(),
        );

        let length_delimited = FramedWrite::new(socket2, LengthDelimitedCodec::new());
        let mut serialized =
            tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default());

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle messages received from the broadcaster and pass them on
                    result = rx.recv() => {
                        let (value, other_addr) = result.unwrap();

                        // Only send the message to other users, don't send to self
                        if other_addr != addr {
                            serialized.send(value).await.unwrap();
                        }
                    }

                    // Messages received from the client
                    result = deserialized.try_next() => {
                        if let Some(msg) = result.unwrap() {
                            match msg {
                                ClientMessage::Ping => {
                                    println!("Got a ping!");
                                    serialized.send(serde_json::to_value(&ServerMessage::PingResponse).unwrap()).await.unwrap();
                                }
                                ClientMessage::ChatMessage { message } => {
                                    let message = serde_json::to_value(&ServerMessage::ChatMessage{ user_number, message }).unwrap();
                                    tx.send((message, addr)).unwrap();
                                }

                                _ => {
                                    println!("Got a message from the client that couldn't be understood")
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
