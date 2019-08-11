#[macro_use]
extern crate futures;

// Poll is a type Alias for Result<Async<T>, E>
use futures::{Async, Future, Poll};
use std::fmt;

struct HelloWorld;

impl Future for HelloWorld {
    type Item = String;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::Ready("hello world".to_string()))
    }
}

struct Display<T>(T);

impl<T> Future for Display<T>
where
    T: Future,
    T::Item: fmt::Display,
{
    type Item = ();
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // let value = match self.0.poll() {
        //     Ok(Async::Ready(value)) => value,
        //     Ok(Async::NotReady) => return Ok(Async::NotReady),
        //     Err(err) => return Err(err),
        // };
        //
        // Same as
        //
        let value = try_ready!(self.0.poll());

        println!("{}", value);
        Ok(Async::Ready(()))
    }
}

fn main() {
    let future = Display(HelloWorld);
    tokio::run(future);
}
