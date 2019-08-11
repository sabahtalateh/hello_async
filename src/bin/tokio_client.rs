//! Hello World Server
//!
//! A simple client that opens a TCP stream, writes "hello world\n", and closes
//! the connection.
//!
//! You can test this out by running:
//!
//!     Linux
//!     ncat -l 6142
//!
//!     Mac
//!     nc -l 6142
//!
//! And then in another terminal run:
//!
//!     cargo run --bin hello_world
//!

use std::net::SocketAddr;
use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

fn main() {
    let addr = "127.0.0.1:6142".parse::<SocketAddr>().unwrap();
    let client = TcpStream::connect(&addr)
        .and_then(|stream| {
            println!("created stream");
            io::write_all(stream, "hello world\n").then(|result| {
                println!("wrote to stream; success={:?}", result.is_ok());
                Ok(())
            })
        })
        .map_err(|err| {
            // All tasks must have an `Error` type of `()`. This forces error
            // handling and helps avoid silencing failures.
            //
            // In our example, we are only going to log the error to STDOUT.
            println!("connection error = {:?}", err);
        });

    println!("About to create the stream and write to it...");
    tokio::run(client);
    println!("Stream has been created and written to.");
}
