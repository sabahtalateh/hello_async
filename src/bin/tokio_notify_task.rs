use futures::Future;
use tokio::prelude::*;

struct Count {
    remaining: usize,
}

impl Future for Count {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        println!("resumed");
        while self.remaining > 0 {
            println!("{}", self.remaining);
            self.remaining -= 1;

            if self.remaining % 10 == 0 {
                println!("paused");
                task::current().notify();
                return Ok(Async::NotReady);
            }
        }
        Ok(Async::Ready(()))
    }
}

fn main() {
    let count = Count {
        remaining: 100,
    };

    tokio::run(count);
}
