use std::io;
use std::mem;

use bytes::BytesMut;
use futures::try_ready;
use tokio::prelude::*;

use super::response;

pub struct WriteAll {
  state: State,
}

enum State {
  Writing {
    a: response::Response,
    buf: BytesMut,
    pos: usize,
  },
  Empty,
}

pub fn write_all(a: response::Response, res: BytesMut) -> WriteAll {
  WriteAll {
    state: State::Writing {
      a,
      buf: res,
      pos: 0,
    },
  }
}

impl Future for WriteAll {
  type Item = response::Response;
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    match self.state {
      State::Writing {
        ref mut a,
        ref buf,
        ref mut pos,
      } => {
        while *pos < buf.len() {
          let n = try_ready!(a.writer.poll_write(&buf[*pos..]));
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
