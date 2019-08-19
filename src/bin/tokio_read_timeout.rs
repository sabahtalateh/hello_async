use futures::{Future, Stream};
use std::time::Duration;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::FutureExt;

fn read_four_bytes(
    socket: TcpStream,
) -> Box<dyn Future<Item = (TcpStream, Vec<u8>), Error = ()> + Send> {
    println!("enter read four bytes");

    let buf = vec![0; 4];
    let fut = io::read_exact(socket, buf)
        .timeout(Duration::from_secs(1))
        .map_err(|_| println!("failed to read 4 bytes by timeout"));
    ;

    Box::new(fut)
}

fn main() {
    let addr = "127.0.0.1:8889".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let server = listener
        .incoming()
        .map_err(|e| eprintln!("error accept = {:?}", e))
        .for_each(|socket| {
            let read_fut = read_four_bytes(socket).and_then(|(_, v)| {
                println!("v = {:?}", v);
                Ok(())
            }).or_else(|_| {
                eprintln!("timeout error");
                Ok(())
            });

            // This can be used to not stop all the program on
            //  timer error in case read future is not
            //  run in the separate task.
            //
            tokio::spawn(read_fut)
        });

    tokio::run(server);
}
