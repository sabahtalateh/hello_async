//! The example coordinates access to a transport over a ping / pong protocol.
//! Pings are sent into the transport and pongs are received.
//! Primary tasks send a message to the coordinator task to initiate a ping,
//! the coordinator task will respond to the ping request with the round trip time.
//! The message sent to the coordinator task over the mpsc contains a oneshot::Sender
//! allowing the coordinator task to respond.
//!
//! https://tokio.rs/docs/futures/spawning/#coordinating-access-to-a-resource
//!
//! Sample Output
//!
//!     cargo run --bin tokio_ping_pong_2
//! [0] entering ping
//! [0] entering recv_pong delay 9.361s
//! [1] entering ping
//! [1] entering recv_pong delay 4.854s
//! [2] entering ping
//! [2] entering recv_pong delay 4.719s
//! [3] entering ping
//! [3] entering recv_pong delay 8.182s
//! [4] entering ping
//! [4] entering recv_pong delay 9.642s
//! [5] entering ping
//! [5] entering recv_pong delay 2.676s
//! [8] entering ping
//! [8] entering recv_pong delay 5.149s
//! [6] entering ping
//! [6] entering recv_pong delay 7.299s
//! [9] entering ping
//! [9] entering recv_pong delay 5.908s
//! [7] entering ping
//! [7] entering recv_pong delay 273ms
//! [7] >>> rtt = 275.67322ms
//! [5] >>> rtt = 2.679343764s
//! [2] >>> rtt = 4.72073866s
//! [1] >>> rtt = 4.855222603s
//! [8] >>> rtt = 5.149836229s
//! [9] >>> rtt = 5.910623536s
//! [6] >>> rtt = 7.301647815s
//! [3] >>> rtt = 8.183006022s
//! [0] >>> rtt = 9.363908809s
//! [4] >>> rtt = 9.644285966s
//!

use futures::future::lazy;
use futures::{future, Future, Sink, Stream};
use rand::Rng;
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::timer::Delay;

const TRANSPORT_MAX_DELAY: Duration = Duration::from_secs(0);
const TRANSPORT_MIN_DELAY: Duration = Duration::from_secs(10);

type Message = (usize, oneshot::Sender<Duration>);

struct Transport {
    min_delay: Duration,
    max_delay: Duration,
}

impl Transport {
    fn new(min_delay: Duration, max_delay: Duration) -> Transport {
        Transport {
            min_delay,
            max_delay,
        }
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
        let wait_millis: u64 =
            rng.gen_range(self.min_delay.as_millis(), self.max_delay.as_millis()) as u64;
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

        let fut = transport
            .recv_pong(&thread_id)
            .map_err(|_| ())
            .and_then(move |_| {
                let rtt = start.elapsed();
                r_tx.send(rtt).unwrap();
                Ok(())
            });
        tokio::spawn(fut)
    })
}

fn rtt(
    thread_id: &usize,
    tx: mpsc::Sender<Message>,
) -> impl Future<Item = (Duration, mpsc::Sender<Message>), Error = ()> {
    let (r_tx, r_rx): (Sender<Duration>, Receiver<Duration>) = oneshot::channel();

    tx.send((*thread_id, r_tx))
        .map_err(|_| ())
        .and_then(|tx| r_rx.map_err(|_| ()).map(|d| (d, tx)))
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
