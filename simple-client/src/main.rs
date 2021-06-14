use simple_server::api::{ClientMessage, ServerMessage};
use futures::prelude::*;
use tokio::net::TcpStream;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[tokio::main]
pub async fn main() {
    let socket = TcpStream::connect("localhost:31194").await.unwrap();

    // Duplicate the socket: one for serializing and one for deserializing
    let socket = socket.into_std().unwrap();
    let socket2 = socket.try_clone().unwrap();
    let socket = TcpStream::from_std(socket).unwrap();
    let socket2 = TcpStream::from_std(socket2).unwrap();

    let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());
    let mut deserialized = tokio_serde::SymmetricallyFramed::new(
        length_delimited,
        SymmetricalJson::<ServerMessage>::default(),
    );

    let length_delimited = FramedWrite::new(socket2, LengthDelimitedCodec::new());
    let mut serialized =
        tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default());

    // Handle incoming messages from the server
    tokio::spawn(async move {
        while let Some(msg) = deserialized.try_next().await.unwrap() {
            match msg {
                ServerMessage::PingResponse => {
                    println!("pong!");
                }
                ServerMessage::ChatMessage{ user_number, message } => {
                    println!("User #{}: {}", user_number, message);
                }
                _ => {
                    println!("Got a message from the server that the client couldn't understand!")
                }
            }
        }
    });

    // Continuously read user input and send appropriate messages to the server
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Error while reading user input!");
        let trimmed = input.trim_matches(char::is_whitespace);
        match trimmed {
            "" => {}
            "/ping" => {
                serialized
                    .send(serde_json::to_value(&ClientMessage::Ping).unwrap())
                    .await
                    .unwrap();
            }
            _ => {
                serialized
                .send(serde_json::to_value(&ClientMessage::ChatMessage{ message: trimmed.to_string() }).unwrap())
                .await
                .unwrap();
            }
        }
    }
}

