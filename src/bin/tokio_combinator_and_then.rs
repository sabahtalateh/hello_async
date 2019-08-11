use futures::Future;
use tokio::io;
use tokio::net::TcpStream;

fn main() {
    let addr = "127.0.0.1:1234".parse().unwrap();

    let future = TcpStream::connect(&addr)
        .and_then(|socket| io::write_all(socket, b"hello world"))
        .map(|_| println!("wirte complete"))
        .map_err(|e| println!("failed - {:?}", e));

    tokio::run(future);
}
