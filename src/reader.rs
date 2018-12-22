use bytes::BytesMut;

use futures::try_ready;
use tokio::net::TcpStream;
use tokio::prelude::*;

use crate::request::Request;

pub struct HttpReader {
  reader: tokio::io::ReadHalf<TcpStream>,
  buffer: BytesMut,
}

impl HttpReader {
  pub fn new(reader: tokio::io::ReadHalf<TcpStream>) -> HttpReader {
    HttpReader {
      reader,
      buffer: BytesMut::new(),
    }
  }

  fn read_buffer(&mut self) -> Poll<(), tokio::io::Error> {
    loop {
      self.buffer.reserve(512); // improve reserve handler
      let n = try_ready!(self.reader.read_buf(&mut self.buffer));

      if n == 0 {
        return Ok(Async::Ready(()));
      }
    }
  }
}

impl Stream for HttpReader {
  type Item = Request;
  type Error = tokio::io::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    let is_closed = self.read_buffer()?.is_ready();

    if is_closed {
      return Ok(Async::Ready(None));
    }

    if !self.buffer.is_empty() {
      match Request::decode(&mut self.buffer).expect("Could not create request") {
        Some(req) => {
          self.buffer.clear();
          return Ok(Async::Ready(Some(req)));
        }
        None => {}
      };
    }

    Ok(Async::NotReady)
  }
}
