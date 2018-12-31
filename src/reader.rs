use std::io;

use futures::try_ready;

use bytes::{BufMut, BytesMut};
use tokio::prelude::*;

use super::request;

pub struct HttpReader<S> {
  socket: S,
  closed: bool,
  buffer: BytesMut,
}

impl<S: AsyncRead> HttpReader<S> {
  pub fn new(socket: S) -> HttpReader<S> {
    HttpReader {
      socket,
      closed: false,
      buffer: BytesMut::with_capacity(1024),
    }
  }
}

impl<S: AsyncRead> Stream for HttpReader<S> {
  type Item = request::Request;
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    // TODO: Add truncate for buffer. Do we actually need that ?
    loop {
      // only if socket is closed complete connection
      if self.closed {
        return Ok(Async::Ready(None));
      }

      // reserve more space only if there is no available space
      if self.buffer.has_remaining_mut() {
        self.buffer.reserve(1024);
      }

      // consider read in separate loop and then parse
      let n = try_ready!(self.socket.read_buf(&mut self.buffer));

      // set socket to close
      if n == 0 {
        self.closed = true;
      }

      if !self.buffer.is_empty() {
        match request::Request::parse(&mut self.buffer) {
          // should we loop again in this place ?
          Ok(Some(req)) => return Ok(Async::Ready(Some(req))),
          // if was not able to parse try to get data again
          Ok(None) => continue,
          Err(e) => return Err(e),
        }
      }
    }
  }
}
