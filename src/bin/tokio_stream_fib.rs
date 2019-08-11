#[macro_use]
extern crate futures;

use futures::{stream, Async, Future, Poll, Stream};
use std::time::Duration;
use std::{fmt, time};
use tokio::timer;
use tokio::timer::Interval;

pub struct Fibonacci {
    interval: timer::Interval,
    curr: u64,
    next: u64,
}

impl Fibonacci {
    fn new(duration: time::Duration) -> Self {
        Fibonacci {
            interval: timer::Interval::new_interval(duration),
            curr: 1,
            next: 1,
        }
    }
}

impl Stream for Fibonacci {
    type Item = u64;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<u64>, ()> {
        // Wait until the next interval
        try_ready!(self
            .interval
            .poll()
            // The interval can fail if the Tokio runtime is unavailable.
            // In this example, the error is ignored.
            .map_err(|_| ()));

        let curr = self.curr;
        let next = curr + self.next;

        self.curr = self.next;
        self.next = next;

        Ok(Async::Ready(Some(curr)))
    }
}

pub struct Display10<T> {
    stream: T,
    curr: usize,
}

impl<T> Display10<T> {
    fn new(stream: T) -> Display10<T> {
        Display10 { stream, curr: 0 }
    }
}

impl<T> Future for Display10<T>
where
    T: Stream,
    T::Item: fmt::Display,
{
    type Item = ();
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while self.curr < 10 {
            let value = match try_ready!(self.stream.poll()) {
                Some(value) => value,
                // There were less than 10 values to display, terminate the
                // future.
                None => break,
            };

            println!("value #{} = {}", self.curr, value);
            self.curr += 1;
        }

        Ok(Async::Ready(()))
    }
}

fn fib() -> impl Stream<Item = u64, Error = ()> {
    stream::unfold((1, 1), |(curr, next)| {
        let yielded = curr;
        let new_next = curr + next;
        let next_state = (next, new_next);
        let fut = futures::future::ok((curr, next_state));

        Some(fut)
    })
}

struct Fib {
    curr: i64,
    next: i64,
}

fn main() {
    let d10 = Display10::new(Fibonacci::new(Duration::from_millis(100)));
    tokio::run(d10);

    let d10 = Display10::new(fib());
    tokio::run(d10);

    let d10 = fib().take(10).for_each(|num| {
        println!("{}", num);
        Ok(())
    });
    tokio::run(d10);

    let mut fib = Fib { curr: 1, next: 1 };
    let future = Interval::new_interval(Duration::from_millis(100)).map(move |_| {
        let curr = fib.curr;
        let next = curr + fib.next;

        fib.curr = fib.next;
        fib.next = next;

        curr
    });
    tokio::run(future.take(10).map_err(|_| ()).for_each(|v| {
        println!("{}", v);
        Ok(())
    }));

    let values = vec!["one", "two", "three"];
    let fut = stream::iter_ok(values).for_each(|v| {
        println!("{}", v);
        Ok(())
    });
    tokio::run(fut);
}
