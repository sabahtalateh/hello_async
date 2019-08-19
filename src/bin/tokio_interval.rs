use tokio::timer::Interval;
use std::time::{Instant, Duration};
use tokio::prelude::*;

fn main() {
    let task = Interval::new(Instant::now(), Duration::from_millis(100))
        .take(10)
        .for_each(|instant| {
            println!("fire; instant={:?}", instant);
            Ok(())
        })
        .map_err(|e| panic!("interval errored; err={:?}", e));

    tokio::run(task);
}
