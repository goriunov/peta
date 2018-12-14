use bytes::BytesMut;

use tokio::net::TcpStream;
use tokio::prelude::*;

use futures::try_ready;

use http::Request;

pub struct Http {
  reader: tokio::io::ReadHalf<TcpStream>,
  buffer: BytesMut,
}

impl Http {
  pub fn new(reader: tokio::io::ReadHalf<TcpStream>) -> Http {
    Http {
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

impl Stream for Http {
  // need to optimize string string case
  type Item = Request<()>;
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
      let parsed = req.parse(&self.buffer[..]).unwrap();

      match parsed {
        httparse::Status::Complete(_) => {
          let request = Request::builder()
            .uri(req.path.unwrap())
            .method(req.method.unwrap())
            .body(())
            .unwrap();

          self.buffer.clear();
          return Ok(Async::Ready(Some(request)));
        }
        httparse::Status::Partial => {}
      };
    }

    return Ok(Async::NotReady);
  }
}
