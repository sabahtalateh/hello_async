#[macro_use]
extern crate futures;

use futures::Future;
use tokio::io;
use tokio::prelude::{Async, AsyncRead, Poll};

// This is going to be our Future.
// In the common case, this is set to Some(Reading),
// but we'll set it to None when we return Async::Ready
// so that we can return the reader and the buffer.
struct ReadExact<R, T>(Option<Reading<R, T>>);

struct Reading<R, T> {
    // This is the stream we're reading from.
    reader: R,
    // This is the buffer we're reading into.
    buffer: T,
    // And this is how far into the buffer we've written.
    pos: usize,
}

// We want to be able to construct a ReadExact over anything
// that implements AsyncRead, and any buffer that can be
// thought of as a &mut [u8].
fn read_exact<R, T>(reader: R, buffer: T) -> ReadExact<R, T>
where
    R: AsyncRead,
    T: AsMut<[u8]>,
{
    ReadExact(Some(Reading {
        reader,
        buffer,
        // Initially, we've read no bytes into buffer.
        pos: 0,
    }))
}

impl<R, T> Future for ReadExact<R, T>
where
    R: AsyncRead,
    T: AsMut<[u8]>,
{
    // When we've filled up the buffer, we want to return both the buffer
    // with the data that we read and the reader itself.
    type Item = (R, T);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0 {
            Some(Reading {
                ref mut reader,
                ref mut buffer,
                ref mut pos,
            }) => {
                let buffer = buffer.as_mut();
                // Check that we haven't finished
                while *pos < buffer.len() {
                    // Try to read data into the remainder of the buffer.
                    // Just like read in std::io::Read, poll_read *can* read
                    // fewer bytes than the length of the buffer it is given,
                    // and we need to handle that by looking at its return
                    // value, which is the number of bytes actually read.
                    //
                    // Notice that we are using try_ready! here, so if poll_read
                    // returns NotReady (or an error), we will do the same!
                    // We uphold the contract that we have arranged to be
                    // notified later because poll_read follows that same
                    // contract, and _it_ returned NotReady.
                    let n = try_ready!(reader.poll_read(&mut buffer[*pos..]));
                    *pos += n;

                    // If no bytes were read, but there was no error, this
                    // generally implies that the reader will provide no more
                    // data (for example, because the TCP connection was closed
                    // by the other side).
                    if n == 0 {
                        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "early eof"));
                    }
                }
            }
            None => panic!("poll on ReadExact after it's done"),
        }

        // We need to return the reader and the buffer, which we can only
        // do by moving them out of self. We do this by taking our state
        // and leaving `None`. This _should_ be fine, because poll()
        // requires callers to not call poll() again after Ready has been
        // returned, so we should only ever see Some(Reading) when poll()
        // is called.
        let reading = self.0.take().expect("must have seen Some above");
        Ok(Async::Ready((reading.reader, reading.buffer)))
    }
}

fn main() {}
