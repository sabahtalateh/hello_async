#[macro_use]
extern crate futures;

use futures::{Future, Poll, Async};
use std::fmt;

//
// Explicit way
//
struct Display<T>(T);

impl<T> Future for Display<T>
where
    T: Future,
    T::Item: fmt::Display,
{
    type Item = ();
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let val = try_ready!(self.0.poll());
        println!("{}", val);
        Ok(Async::Ready(()))
    }
}

struct HelloWorld;

impl Future for HelloWorld {
    type Item = String;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::Ready("hello world".to_string()))
    }
}

fn main() {
    let future = Display(HelloWorld);
    tokio::run(future);

    // Same as before
    let future = HelloWorld.map(|value| {
        println!("{}", value);
    });
    tokio::run(future);
}
