# Simple Tokio Server

This is a basic example of a server and client written in pure Rust and
communicating using TCP through an API defined by the server library. The
network connections are handled using Tokio, and all messages between the
server and the client are serialized using Serde before being sent. This
particular example defines a chat server with a basic API consisting of
chat messages and pings, but it is easy to create your own API by adding
items to the enums in `api.rs`. Note that the client and server code are
completely independent of each other, and any client only needs to import
the server library to access the API.
