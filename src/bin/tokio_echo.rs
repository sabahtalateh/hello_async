//! Start the echo server
//!
//! Use netcat to send tcp packets to server
//!
//!     nc localhost 9876
//!     Hello World!
//!
//! Use `Ctrl+C` to close the connection
//!

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

fn main() {
    // Bind the server's socket
    let addr = "127.0.0.1:9876".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

    // Convert the `TcpListener` to a stream of incoming connections
    //  with the `incoming` method. We then define how to process each element in
    //  the stream with the `for_each` combinator
    let server = listener
        .incoming()
        .for_each(|socket| {
            // Split the socket into readable and writable parts
            let (reader, writer) = socket.split();
            // Copy bytes from the reader into the writer
            let amount = io::copy(reader, writer);

            let msg = amount.then(|result| {
                match result {
                    Ok(amount, _, _) => println!("wrote {} bytes", amount),
                    Err(e) => println!("error: {}", e),
                }

                Ok(())
            });

            // Spawn the task that handles the client connection socket on to the
            // tokio runtime. This means each client connection will be handled
            // concurrently
            tokio::spawn(msg);
            Ok(())
        })
        .map_err(|err| {
            // Handle error by printing it to STDOUT
        });

    println!("server running on localhost:9876");

    // Start the server
    //
    // This does a few things
    //
    // * Start the tokio runtime
    // * Spawns the `server` task onto the runtime
    // * Block the current thread until the runtime becomes idle, i.e.
    //   all spawned tasks have completed
    tokio::run(server);

    //    // Pull out a stream of sockets for incoming connections
    //    let server = listener
    //        .incoming()
    //        .map_err(|e| eprintln!("accept failed = {:?}", e))
    //        .for_each(|sock| {
    //            // Split up reading and writing parts of socket
    //            let (reader, writer) = sock.split();
    //
    //            // A futures that echos data and return how many bytes
    //            //  were copied..
    //            let bytes_copied = copy(reader, writer);
    //
    //            // .. after which we print what happened
    //            let handle_conn = bytes_copied
    //                .map(|amt| {
    //                    println!("wrote {:} bytes", amt.0);
    //                })
    //                .map_err(|err| {
    //                    eprintln!("IO error {:?}", err);
    //                });
    //
    //            // Spawn a future as a concurrent task
    //            tokio::spawn(handle_conn)
    //        });
    //
    //    // Start the Tokio runtime
    //    tokio::run(server);
}
