use futures::{Future, Stream};
use tokio::io;
use tokio::net::TcpListener;

fn main() {
    let addr = "127.0.0.1:9878".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    tokio::run({
        listener
            .incoming()
            .for_each(|socket| {
                // An inbound socket has been received.
                //
                // Spawn a new task to process the socket
                tokio::spawn({
                    // In this example, "hello world" will be written to the
                    // socket followed by the socket being closed.
                    io::write_all(socket, "hello world")
                        // Drop the socket
                        .map(|_| ())
                        // Write any error to STDOUT
                        .map_err(|e| println!("socker error = {:?}", e))
                });

                // Receive the next inbound socket
                Ok(())
            })
            .map_err(|e| {
                println!("socket error = {}", e);
            })
    });
}
