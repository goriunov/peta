use std::io;
use std::mem;

use bytes::BytesMut;
use futures::{try_ready, Future, Poll};
use tokio::prelude::*;

pub struct WriteAll<A> {
  state: State<A>,
}

enum State<A> {
  Writing { a: A, buf: BytesMut, pos: usize },
  Empty,
}

pub fn write_all<A: AsyncWrite>(a: A, res: BytesMut) -> WriteAll<A> {
  WriteAll {
    state: State::Writing {
      a,
      buf: res,
      pos: 0,
    },
  }
}

impl<A: AsyncWrite> Future for WriteAll<A> {
  type Item = A;
  type Error = io::Error;

  fn poll(&mut self) -> Poll<A, io::Error> {
    match self.state {
      State::Writing {
        ref mut a,
        ref buf,
        ref mut pos,
      } => {
        while *pos < buf.len() {
          let n = try_ready!(a.poll_write(&buf[*pos..]));
          *pos += n;
          if n == 0 {
            return Err(io::Error::new(
              io::ErrorKind::WriteZero,
              "zero-length write",
            ));
          }
        }
      }
      State::Empty => panic!("poll a WriteAll after it's done"),
    }

    match mem::replace(&mut self.state, State::Empty) {
      State::Writing { a, .. } => Ok(a.into()),
      State::Empty => panic!(),
    }
  }
}
