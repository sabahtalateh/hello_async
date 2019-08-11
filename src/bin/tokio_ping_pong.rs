//! The example coordinates access to a transport over a ping / pong protocol.
//! Pings are sent into the transport and pongs are received.
//! Primary tasks send a message to the coordinator task to initiate a ping,
//! the coordinator task will respond to the ping request with the round trip time.
//! The message sent to the coordinator task over the mpsc contains a oneshot::Sender
//! allowing the coordinator task to respond.
//!
//! https://tokio.rs/docs/futures/spawning/#coordinating-access-to-a-resource
//!

use futures::future::lazy;
use futures::{future, Future, Sink, Stream};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

type Message = oneshot::Sender<Duration>;

struct Transport;

impl Transport {
    fn send_ping(&self) {
        println!("entering ping");
    }

    fn recv_pong(&self) -> impl Future<Item = (), Error = io::Error> {
        println!("entering recv_pong");
        future::ok::<(), io::Error>(())
    }
}

fn coordinator_task(rx: mpsc::Receiver<Message>) -> impl Future<Item = (), Error = ()> {
    //    let transport = Transport;

    //    rx.map_err(|_| ()).for_each(move |pong_tx| {
    //        let start = Instant::now();
    //
    //        transport.send_ping();
    //
    //        transport.recv_pong().map_err(|_| ()).and_then(move |_| {
    //            let rtt = start.elapsed();
    //            pong_tx.send(rtt).unwrap();
    //            Ok(())
    //        })
    //    })
    future::ok(())
}

fn rtt(
    tx: mpsc::Sender<Message>,
) -> impl Future<Item = (Duration, mpsc::Sender<Message>), Error = ()> {
    let (resp_tx, resp_rx) = oneshot::channel();

    tx.send(resp_tx)
    //    tx.send(resp_tx)
    //        .and_then(|tx| {
    //            resp_rx.map(|dur| (dur, tx)).map_err(|_| ())
    //        })
    //        .map_err(|_| ())
}

///// Request an rtt.
//fn rtt(
//    tx: mpsc::Sender<Message>,
//) -> impl Future<Item = (Duration, mpsc::Sender<Message>), Error = ()> {
//    let (resp_tx, resp_rx) = oneshot::channel();
//
//    tx.send(resp_tx)
//        .map_err(|_| ())
//        .and_then(|tx| resp_rx.map(|dur| (dur, tx)).map_err(|_| ()))
//}

fn main() {
    tokio::run(lazy(|| {
        // Create the channel that is used to communicate with the
        // background task.
        let (tx, rx) = mpsc::channel(1024);

        // Spawn the background task:
        tokio::spawn(coordinator_task(rx));

        // Spawn a few tasks that use the coordinator to requst RTTs.
        for _ in 0..4 {
            let tx = tx.clone();

            tokio::spawn(lazy(|| rtt(tx)))
        }

        Ok(())
    }));
    //    // Start the application
    //    tokio::run(lazy(|| {
    //        // Create the channel that is used to communicate with the
    //        // background task.
    //        let (tx, rx) = mpsc::channel(1_024);
    //
    //        // Spawn the background task:
    //        tokio::spawn(coordinator_task(rx));
    //
    //        // Spawn a few tasks that use the coordinator to requst RTTs.
    //        for _ in 0..4 {
    //            let tx = tx.clone();
    //
    //            tokio::spawn(lazy(|| {
    //                rtt(tx).and_then(|(dur, _)| {
    //                    println!("duration = {:?}", dur);
    //                    Ok(())
    //                })
    //            }));
    //        }
    //
    //        Ok(())
    //    }));
}
