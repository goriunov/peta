use bytes::BytesMut;

use tokio::net::TcpStream;
use tokio::prelude::*;

use futures::try_ready;

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
      // need to optimize http parser and hashmap
      let mut headers = [httparse::EMPTY_HEADER; 16];
      let mut req = httparse::Request::new(&mut headers);

      // handle things
      let parsed = req
        .parse(&self.buffer[..])
        .expect("Could not parse http request");

      match parsed {
        httparse::Status::Complete(_) => {
          let req = Request::new(
            req.path.expect("Could not get path").to_string(),
            req.method.expect("Could not get method").to_string(),
          );

          self.buffer.clear();
          return Ok(Async::Ready(Some(req)));
        }
        httparse::Status::Partial => {}
      };
    }

    return Ok(Async::NotReady);
  }
}
