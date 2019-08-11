//! Run netcat to accept connections
//!
//!     nc -l 1234
//!
extern crate bytes;
#[macro_use]
extern crate futures;

use bytes::{Buf, Bytes};
use futures::{Async, Future, Poll};
use std::io::{self, Cursor};
use tokio::io::AsyncWrite;
use tokio::net::{tcp::ConnectFuture, TcpStream};

enum HelloWorld {
    Connecting(ConnectFuture),
    Connected(TcpStream, Cursor<Bytes>),
}

impl Future for HelloWorld {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::HelloWorld::*;
        loop {
            println!("loop");
            match self {
                Connecting(ref mut f) => {
                    println!("connecting");
                    let socket = try_ready!(f.poll());
                    let data = Cursor::new(Bytes::from_static(b"Hello World\n"));
                    *self = Connected(socket, data);
                }
                Connected(ref mut socket, ref mut data) => {
                    println!("connected");
                    // Keep trying to write the buffer to the socket as long as the
                    //  buffer has more bytes available for consumption
                    while data.has_remaining() {
                        try_ready!(socket.write_buf(data));
                    }
                    return Ok(Async::Ready(()));
                }
            }
        }
    }
}

fn main() {
    let addr = "127.0.0.1:1234".parse().unwrap();
    let connect_ftr = TcpStream::connect(&addr);
    // Map the error since tokio::run expects a Future<Item=(), Error=()>
    let hello_world_ftr = HelloWorld::Connecting(connect_ftr).map_err(|e| {
        println!("{}", e);
    });

    tokio::run(hello_world_ftr);
}
