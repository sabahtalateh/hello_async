use futures::future::lazy;
use futures::sync::{mpsc, oneshot};
use futures::{stream, Future, Sink, Stream};

fn main() {
    tokio::run(lazy(|| {
        for i in 0..4 {
            tokio::spawn(lazy(move || {
                println!("Hello from task {}", i);
                Ok(())
            }));
        }
        Ok(())
    }));

    tokio::run(lazy(|| {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(lazy(|| {
            tx.send("hello from spawned task");
            Ok(())
        }));

        rx.and_then(|msg| {
            println!("Got '{}'", msg);
            Ok(())
        })
        .map_err(|e| {
            println!("{}", e);
        })
    }));

    tokio::run(lazy(|| {
        let (tx, rx) = mpsc::channel(1_024);

        tokio::spawn({
            stream::iter_ok(0..10)
                .fold(tx, |tx, i| {
                    tx.send(format!("Message {} from spawned task", i))
                        .map_err(|e| println!("error = {:?}", e))
                })
                .map(|_| ()) // Drop tx handle
        });

        rx.for_each(|msg| {
            println!("Got `{}`", msg);
            Ok(())
        })
    }));
}
