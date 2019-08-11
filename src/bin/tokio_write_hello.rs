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
                    let data = Cursor::new(Bytes::from_static(b"But I must explain to you how all this mistaken idea of denouncing pleasure and praising pain was born and I will give you a complete account of the system, and expound the actual teachings of the great explorer of the truth, the master-builder of human happiness. No one rejects, dislikes, or avoids pleasure itself, because it is pleasure, but because those who do not know how to pursue pleasure rationally encounter consequences that are extremely painful. Nor again is there anyone who loves or pursues or desires to obtain pain of itself, because it is pain, but because occasionally circumstances occur in which toil and pain can procure him some great pleasure. To take a trivial example, which of us ever undertakes laborious physical exercise, except to obtain some advantage from it? But who has any right to find fault with a man who chooses to enjoy a pleasure that has no annoying consequences, or one who avoids a pain that produces no resultant pleasure?\n"));
                    *self = Connected(socket, data);
                }
                Connected(ref mut socket, ref mut data) => {
                    println!("connected");
                    // Keep trying to write the buffer to the socket as long as the
                    //  buffer has more bytes available for consumption
                    while data.has_remaining() {
                        println!("writing");
                        let bytes_wrote = try_ready!(socket.write_buf(data));
                        println!("wrote {} bytes", bytes_wrote);
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
