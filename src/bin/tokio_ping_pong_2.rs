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
use rand::Rng;
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::timer::Delay;

const TRANSPORT_MAX_DELAY: Duration = Duration::from_secs(1);

type Message = oneshot::Sender<Duration>;

struct Transport {
    max_delay: Duration,
}

impl Transport {
    fn new(max_delay: Duration) -> Transport {
        Transport { max_delay }
    }

    fn default() -> Transport {
        Transport {
            max_delay: TRANSPORT_MAX_DELAY,
        }
    }

    fn send_ping(&self) {
        println!("entering ping");
    }

    fn recv_pong(&self) -> impl Future<Item = (), Error = io::Error> {
        println!("entering recv_pong");

        let mut rng = rand::thread_rng();
        let wait_millis: u64 = rng.gen_range(0, self.max_delay.as_millis()) as u64;
        let delay = Duration::from_millis(wait_millis);

        println!("Delay {:?}", delay);

        let when = Instant::now() + delay;

        Delay::new(when)
            .and_then(|_| Ok(()))
            .map_err(|e| panic!("delay errored; err={:?}", e))
    }
}

fn coordinator_task(rx: mpsc::Receiver<Message>) -> impl Future<Item = (), Error = ()> {
    let transport = Transport::default();

    rx.map_err(|_| ()).for_each(move |r_tx| {
        let start = Instant::now();

        transport.send_ping();

        transport.recv_pong().map_err(|_| ()).and_then(move |_| {
            let rtt = start.elapsed();
            r_tx.send(rtt).unwrap();
            Ok(())
        })
    })
}

fn rtt(tx: mpsc::Sender<Message>) -> impl Future<Item = (Duration, mpsc::Sender<Message>), Error = ()> {
    let (r_tx, r_rx) = oneshot::channel();

    tx.send(r_tx)
        .map_err(|_| ())
        .and_then(|tx| {
            r_rx
                .map_err(|_| ())
                .map(|d| (d, tx))
        })
}

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

            tokio::spawn(lazy(|| {
                rtt(tx).and_then(|(d, _)| {
                    println!("{:?}", d);
                    Ok(())
                })
            }));
        }

        Ok(())
    }));
}
