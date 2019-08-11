use futures::future::FutureResult;
use futures::{future, stream, Future, Stream};
use std::io;
use std::sync;

type Data = String;

fn get_data() -> Result<Data, io::Error> {
    Ok("Hello".to_string())
}

fn get_ok_data() -> Result<Vec<Data>, io::Error> {
    let mut dst = vec![];

    for _ in 0..10 {
        get_data().and_then(|data| {
            dst.push(data);
            Ok(())
        });
    }

    Ok(dst)
}

fn get_data_fp_style() -> impl Future<Item = Data, Error = io::Error> {
    future::ok("Hello".to_string())
}

fn get_ok_data_fp_style() -> impl Future<Item = Vec<Data>, Error = io::Error> {
    let dst = vec![];

    // Start with an unbounded stream that uses unit values.
    stream::repeat(())
        // Only take 10. This is how the for loop is simulated using a functional style.
        .take(10)
        // The `fold` combinator is used here because, in order to be
        // functional, the state must be moved into the combinator. In this
        // case, the state is the `dst` vector.
        .fold(dst, move |mut dst, _| {
            // Once again, the `dst` vector must be moved into the nested
            // closure.
            get_data_fp_style().and_then(move |item| {
                dst.push(item);
                Ok(dst)
            })
        })
}

fn get_message() -> impl Future<Item = Data, Error = io::Error> {
    future::ok("shmyak".to_string())
}

fn print_multi() -> impl Future<Item = (), Error = io::Error> {
    let dst = sync::Arc::new("carl".to_string());

    let futures: Vec<_> = (0..10)
        .map(|_| {
            // Clone the `name` handle, this allows multiple concurrent futures
            // to access the name to print.
            let name = sync::Arc::clone(&dst);

            get_message().and_then(move |message| {
                println!("Hello {} {}", name, message);
                Ok(())
            })
        })
        .collect();

    future::join_all(futures).map(|_| ())
}

fn with_future<F: Future<Item = String, Error = io::Error>>(f: F) {}

fn add_10<F>(f: F) -> impl Future<Item = i32, Error = F::Error>
where
    F: Future<Item = i32>,
{
    f.map(|i| i + 10)
}

fn get_message1() -> impl Future<Item = Data, Error = &'static str> {
    future::ok("shmyak".to_string())
}

fn my_op(arg: String) -> Box<dyn Future<Item = String, Error = &'static str> + Send> {
    if arg == "foo" {
        return Box::new(get_message1().map(|m| format!("FOO = {}", m)));
    }

    Box::new(future::err("something went wrong"))
}

fn main() {
    println!("{:?}", get_ok_data());

    let future = get_ok_data_fp_style()
        .map(|v| {
            println!("{:?}", v);
        })
        .map_err(|e| {
            println!("{}", e);
        });
    tokio::run(future);

    let future = print_multi().map_err(|e| {
        println!("{}", e);
    });
    tokio::run(future);

    let future = get_message();
    with_future(future);

    let future: FutureResult<i32, ()> = future::ok(1);
    let future = add_10(future).map(|r| {
        println!("{}", r);
    });
    tokio::run(future);

    let future = my_op("foo".to_string())
        .map(|m| {
            println!("{}", m);
        })
        .map_err(|e| {
            println!("{:?}", e);
        });
    tokio::run(future);
}
