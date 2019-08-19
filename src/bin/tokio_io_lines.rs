//! Use netcat as a server to run this example
//!
//!     cat file | nc -l 12345

use tokio::net::tcp::TcpStream;
use tokio::prelude::*;

fn main() {
    let addr = "127.0.0.1:12345".parse().unwrap();
    let lines_fut = TcpStream::connect(&addr).and_then(|stream| {
        // We want to parse out each line we receive on stream.
        // To do that, we may need to buffer input for a little while
        // (if the server sends two lines in one packet for example).
        // Because of that, lines requires that the AsyncRead it is
        // given *also* implements BufRead. This may be familiar if
        // you've ever used the lines() method from std::io::BufRead.
        // Luckily, BufReader from the standard library gives us that!
        let stream = std::io::BufReader::new(stream);
        tokio::io::lines(stream).for_each(|line| {
            println!("server sent us the line: {}", line);
            // This closure is called for each line we receive,
            // and returns a Future that represents the work we
            // want to do before accepting the next line.
            // In this case, we just wanted to print, so we
            // don't need to do anything more.
            Ok(())
        })
    });

    tokio::run(lines_fut.map(|_| ()).map_err(|e| eprintln!("{:?}", e)))
}
