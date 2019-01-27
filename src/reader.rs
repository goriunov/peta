use std::io;
use tokio::prelude::*;

use bytes::{BufMut, BytesMut};

use crate::request;

pub enum ReturnState {
  Full,
  Chunked,
}

enum ReaderState {
  Body,
  Chunk,
  Headers,
}

pub struct Reader<S> {
  req: Option<request::Request>,
  state: ReaderState,
  socket: S,
  buffer: BytesMut,
  chunked: bool,
  body_size: usize,
}

impl<S: AsyncRead> Reader<S> {
  pub fn new(socket: S) -> Reader<S> {
    Reader {
      req: None,
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
  type Item = (request::Request, ReturnState);
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    loop {
      match self.state {
        ReaderState::Body => {
          if self.buffer.len() >= self.body_size {
            // we are ready to start next request
            self.state = ReaderState::Headers;
            self.body_size = 0;

            // get current request and stream it out
            let mut request = std::mem::replace(&mut self.req, None).unwrap();
            request.data = Some(self.buffer.split_to(self.body_size));
            return Ok(Async::Ready(Some((request, ReturnState::Full))));
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

              if !self.chunked {
                self.body_size = self.body_size + amt;

                if self.buffer.len() >= self.body_size {
                  // request is ready
                  let request = request::Request {
                    headers,
                    method: self.to_slice(r.method.unwrap().as_bytes()),
                    body: (amt, self.body_size),
                    data: Some(self.buffer.split_to(self.body_size)),
                  };

                  // reset on completed request
                  self.body_size = 0;
                  return Ok(Async::Ready(Some((request, ReturnState::Full))));
                }

                self.state = ReaderState::Body;
                self.req = Some(request::Request {
                  headers,
                  method: self.to_slice(r.method.unwrap().as_bytes()),
                  body: (amt, self.body_size),
                  data: None,
                });
              } else {
                self.state = ReaderState::Chunk;
                // we need to stream request as ready
                // and on next iteration it will check if we have any chunks available
              }
            }
            Ok(httparse::Status::Partial) => {
              // continue reading more
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

      match self.socket.read_buf(&mut self.buffer) {
        Ok(Async::Ready(0)) => {
          // connection is dead :)
          return Ok(Async::Ready(None));
        }
        Ok(Async::Ready(_)) => continue,
        Ok(Async::NotReady) => {
          return Ok(Async::NotReady);
        }
        Err(e) => {
          return Err(e);
        }
      }
    }
  }
}
