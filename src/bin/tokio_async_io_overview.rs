use futures::{Future, Stream};
use tokio::io::AsyncRead;
use tokio::net::TcpListener;

fn main() {
    let addr = "127.0.0.1:7777".parse().unwrap();

    // Set up a listening socket, just like in std::net
    let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

    // Listen for incoming connections.
    // This is similar to the iterator of incoming connections that
    // .incoming() from std::net::TcpListener, produces, except that
    // it is an asynchronous Stream of tokio::net::TcpStream instead
    // of an Iterator of std::net::TcpStream.
    let incoming = listener.incoming();

    // Since this is a Stream, not an Iterator, we use the for_each
    // combinator to specify what should happen each time a new
    // connection becomes available.
    let server = incoming
        .map_err(|e| eprintln!("accept failed = {}", e))
        .for_each(|socket| {
            // Each time we get a connection, this closure gets called.
            // We want to construct a Future that will read all the bytes
            // from the socket, and write them back on that same socket.
            //
            // If this were a TcpStream from the standard library, a read or
            // write here would block the current thread, and prevent new
            // connections from being accepted or handled. However, this
            // socket is a Tokio TcpStream, which implements non-blocking
            // I/O! So, if we read or write from this socket, and the
            // operation would block, the Future will just return NotReady
            // and then be polled again in the future.
            //
            // While we *could* write our own Future combinator that does an
            // (async) read followed by an (async) write, we'll instead use
            // tokio::io::copy, which already implements that. We split the
            // TcpStream into a read "half" and a write "half", and use the
            // copy combinator to produce a Future that asynchronously
            // copies all the data from the read half to the write half.
            let (reader, writer) = socket.split();
            let bytes_copied = tokio::io::copy(reader, writer);
            let handle_conn = bytes_copied
                .map(|amt| {
                    println!("wrote {:?} bytes", amt);
                })
                .map_err(|e| {
                    eprintln!("I/O error {:?}", e);
                });

            // handle_conn here is still a Future, so it hasn't actually
            // done any work yet. We *could* return it here; then for_each
            // would wait for it to complete before it accepts the next
            // connection. However, we want to be able to handle multiple
            // connections in parallel, so we instead spawn the future and
            // return an "empty" future that immediately resolves so that
            // Tokio will _simultaneously_ accept new connections and
            // service this one.
            tokio::spawn(handle_conn)
        });

    tokio::run(server);
}
