use futures::future;
use futures::Future;

// Ideally impl Trait will prevent us from needing to be aware of FutureResult.
type ExampleFuture = future::FutureResult<usize, ExampleFutureError>;

#[derive(Debug, PartialEq)]
pub enum ExampleFutureError {
    Oops,
}

pub fn new_example_future() -> ExampleFuture {
    futures::future::ok(2)
}

pub fn new_example_future_err() -> ExampleFuture {
    futures::future::err(ExampleFutureError::Oops)
}

fn main() {
    assert_eq!(new_example_future().wait(), Ok(2));
}
