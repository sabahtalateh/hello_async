//! Use netcat as a server to run this example
//!
//!     cat file | nc -l 12345

use tokio::net::tcp::TcpStream;
use tokio::prelude::*;

fn main() {
    let addr = "127.0.0.1:12345".parse().unwrap();
    let read_8_fut = TcpStream::connect(&addr)
        .and_then(|stream| {
            // We need to create a buffer for read_exact to write into.
            // A Vec<u8> is a good starting point.
            // read_exact will read buffer.len() bytes, so we need
            // to make sure the Vec isn't empty!
            let mut buf = vec![0; 8];

            // read_exact returns a Future that resolves when
            // buffer.len() bytes have been read from stream.
            tokio::io::read_exact(stream, buf)
        })
        .and_then(|(stream, buf)| {
            // Then, we use write_all to write the entire buffer back:
            tokio::io::write_all(stream, buf)
        })
        .inspect(|(_stream, buf)| {
            // Notice that we get both the buffer and the stream back
            // here, so that we can now continue using the stream to
            // send a reply for example.
            println!("got eight bytes: {:x?}", buf);
        });

    tokio::run(read_8_fut.map(|_| ()).map_err(|e| eprintln!("{:?}", e)))
}
