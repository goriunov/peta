use futures::try_ready;
use std::io;
use tokio::prelude::*;

use bytes::{BufMut, BytesMut};

use crate::request;

pub enum ReturnType {
  Data(BytesMut, bool),
  Request(request::Request),
}

enum ReaderState {
  Body,
  Chunk,
  Headers,
}

pub struct Reader<S> {
  state: ReaderState,
  socket: S,
  buffer: BytesMut,
  chunked: bool,
  body_size: usize,
}

impl<S: AsyncRead> Reader<S> {
  pub fn new(socket: S) -> Reader<S> {
    Reader {
      socket,
      state: ReaderState::Headers,
      buffer: BytesMut::with_capacity(1024),
      chunked: false,
      body_size: 0,
    }
  }

  pub fn to_slice(&self, a: &[u8]) -> request::Slice {
    let start = a.as_ptr() as usize - self.buffer.as_ptr() as usize;
    (start, start + a.len())
  }
}

impl<S: AsyncRead> Stream for Reader<S> {
  type Item = ReturnType;
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    loop {
      match self.state {
        ReaderState::Body => {
          if self.buffer.len() >= self.body_size {
            // we are ready to start next request
            let data = self.buffer.split_to(self.body_size);
            self.state = ReaderState::Headers;
            self.body_size = 0;

            return Ok(Async::Ready(Some(ReturnType::Data(data, true))));
          }
        }
        ReaderState::Chunk => {
          // parse chunk properly
        }
        ReaderState::Headers => {
          let mut headers = [httparse::EMPTY_HEADER; 50];
          let mut r = httparse::Request::new(&mut headers);

          let status = r
            .parse(&self.buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, "Could not parse request"));

          match status {
            Ok(httparse::Status::Complete(amt)) => {
              let mut headers: Vec<(String, request::Slice)> = Vec::with_capacity(r.headers.len());
              for header in r.headers.iter() {
                let header_name = header.name.to_lowercase();

                if header_name == "transfer-ecoding" {
                  // we need to check if it is actually chunked below thing is not right
                  self.chunked = true
                }

                if header_name == "content-length" {
                  self.body_size = std::str::from_utf8(header.value)
                    .expect("Wrong value in header")
                    .parse::<usize>()
                    .expect("Could not parse usize");
                }

                headers.push((header_name, self.to_slice(header.value)));
              }

              self.state = ReaderState::Body;
              if self.chunked {
                // and on next iteration it will check if we have any chunks available
                self.state = ReaderState::Chunk;
              }

              let req = request::Request {
                headers,
                method: self.to_slice(r.method.unwrap().as_bytes()),
                data: self.buffer.split_to(amt),
              };

              return Ok(Async::Ready(Some(ReturnType::Request(req))));
            }
            Ok(httparse::Status::Partial) => {
              // continue reading as no enough headers available
            }
            Err(e) => {
              return Err(e);
            }
          };
        }
      }

      if !self.buffer.has_remaining_mut() {
        self.buffer.reserve(1024);
      }

      let n = try_ready!(self.socket.read_buf(&mut self.buffer));
      if n == 0 {
        return Ok(Async::Ready(None));
      }
    }
  }
}
