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
use tokio::sync::oneshot::{Sender, Receiver};

const TRANSPORT_MAX_DELAY: Duration = Duration::from_secs(0);
const TRANSPORT_MIN_DELAY: Duration = Duration::from_secs(10);

type Message = (usize, oneshot::Sender<Duration>);

struct Transport {
    min_delay: Duration,
    max_delay: Duration,
}

impl Transport {
    fn new(min_delay: Duration, max_delay: Duration) -> Transport {
        Transport { min_delay, max_delay }
    }

    fn default() -> Transport {
        Transport::new(TRANSPORT_MAX_DELAY, TRANSPORT_MIN_DELAY)
    }

    fn send_ping(&self, thread_id: &usize) {
        println!("[{}] entering ping", thread_id);
    }

    fn recv_pong(&self, thread_id: &usize) -> impl Future<Item = (), Error = io::Error> {
        print!("[{}] entering recv_pong", thread_id);

        let mut rng = rand::thread_rng();
        let wait_millis: u64 = rng.gen_range(self.min_delay.as_millis(), self.max_delay.as_millis()) as u64;
        let delay = Duration::from_millis(wait_millis);

        println!(" delay {:?}", delay);

        let when = Instant::now() + delay;

        Delay::new(when)
            .and_then(|_| Ok(()))
            .map_err(|e| panic!("delay errored; err={:?}", e))
    }
}

fn coordinator_task(rx: mpsc::Receiver<Message>) -> impl Future<Item = (), Error = ()> {
    let transport = Transport::default();

    rx.map_err(|_| ()).for_each(move |(thread_id, r_tx)| {
        let start = Instant::now();

        transport.send_ping(&thread_id);

        let fut = transport.recv_pong(&thread_id).map_err(|_| ()).and_then(move |_| {
            let rtt = start.elapsed();
            r_tx.send(rtt).unwrap();
            Ok(())
        });
        tokio::spawn(fut)
    })
}

fn rtt(thread_id: &usize, tx: mpsc::Sender<Message>) -> impl Future<Item = (Duration, mpsc::Sender<Message>), Error = ()> {
    let (r_tx, r_rx) : (Sender<Duration>, Receiver<Duration>) = oneshot::channel();

    tx.send((*thread_id, r_tx))
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
        for id in 0..10 {
            let tx = tx.clone();

            tokio::spawn(lazy(move || {
                rtt(&id, tx).and_then(move |(d, _)| {
                    println!("[{}] >>> rtt = {:?}", id, d);
                    Ok(())
                })
            }));
        }

        Ok(())
    }));
}
