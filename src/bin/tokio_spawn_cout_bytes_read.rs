use futures::sync::mpsc;
use futures::{future, future::lazy, stream, Future, Sink, Stream};
use std::time::Duration;
use tokio::io;
use tokio::net::TcpListener;
use tokio::timer::Interval;

// Defines the background task. The `rx` argument is the channel receive
// handle. The task will pull `usize` values (which represent number of
// bytes read by a socket) off the channel and sum it internally. Every
// 30 seconds, the current sum is written to STDOUT and the sum is reset
// to zero.
fn bg_task(rx: mpsc::Receiver<usize>) -> impl Future<Item = (), Error = ()> {
    // The stream of received `usize` values will be merged with a 30
    // second interval stream. The value types of each stream must
    // match. This enum is used to track the various values.
    #[derive(Eq, PartialEq, Debug)]
    enum Item {
        Value(usize),
        Tick,
        Done,
    };

    // Interval at which the current sum is written to STDOUT.
    let tick_dur = Duration::from_secs(30);
    let interval = Interval::new_interval(tick_dur)
        .map(|_| Item::Tick)
        .map_err(|_| ());

    // Turn the stream into a sequence of:
    // Item(num), Item(num), ... Done
    let items = rx
        .map(Item::Value)
        // Merge in the stream of intervals
        .select(interval)
        // Terminate the stream once `Done` is received. This is necessary
        // because `Interval` is an infinite stream and `select` will keep
        // selecting on it.
        .chain(stream::once(Ok(Item::Done)))
        .take_while(|item| future::ok(*item != Item::Done));

    // With the stream of `Item` values, start our logic.
    //
    // Using `fold` allows the state to be maintained across iterations.
    // In this case, the state is the number of read bytes between tick.
    items
        .fold(0, |sum, item| {
            match item {
                // Sum the number of bytes with the state.
                Item::Value(v) => future::ok(sum + v),
                Item::Tick => {
                    println!("bytes read = {}", sum);
                    future::ok(0)
                }
                _ => unreachable!(),
            }
        })
        .map(|_| ())
}

fn main() {
    tokio::run(lazy(|| {
        let addr = "127.0.0.1:9876".parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        // Create the channel that is used to communicate with the
        // background task.
        let (tx, rx) = mpsc::channel::<usize>(1);

        // Spawn the background task:
        tokio::spawn(bg_task(rx));

        listener
            .incoming()
            .for_each(move |socket| {
                // An inbound socket has been received.
                //
                // Spawn a new task to process the socket
                tokio::spawn({
                    // Each spawned task will have a clone of the sender handle.
                    let tx = tx.clone();

                    // In this example, all bytes read from the
                    // socket will be placed into a Vec.
                    io::read_to_end(socket, vec![])
                        // Drop the socket
                        .and_then(move |(_, buf)| {
                            tx.send(buf.len())
                                .map_err(|e| io::ErrorKind::Other.into())
                        })
                        .map(|_| ())
                        // Write any error to STDOUT
                        .map_err(|e| println!("socket error = {}", e))
                });

                // Receive the next inbound socket
                Ok(())
            })
            .map_err(|e| println!("listener error = {}", e))
    }))
}
