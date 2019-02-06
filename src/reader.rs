use bytes::{BufMut, Bytes, BytesMut};
use tokio::prelude::*;

use super::request;
use super::response;
use super::router;

type Writer = tokio::io::WriteHalf<tokio::net::TcpStream>;
type ReturnFuture = Box<
  dyn Future<Item = ((request::Request, response::Response)), Error = std::io::Error> + Send + Sync,
>;

enum ReaderState {
  Body,
  Chunk,
  Headers,
}

enum FutureProcessState {
  Empty,
  Ready((request::Request, response::Response)),
  Processing(ReturnFuture),
}

pub struct Reader<S, T> {
  socket: S,
  buffer: BytesMut,
  body_size: usize,
  read_state: ReaderState,
  future_state: FutureProcessState,
  router_raw_pointer: *const T,
}

impl<S, T> Reader<S, T> {
  pub fn new(socket: S, writer: Writer, router: &T) -> Reader<S, T>
  where
    T: router::FunctionCall,
  {
    Reader {
      socket,
      read_state: ReaderState::Headers,
      body_size: 0,
      buffer: BytesMut::with_capacity(1024),
      router_raw_pointer: router as *const T,
      future_state: FutureProcessState::Ready((
        request::Request::new(),
        response::Response::new(writer),
      )),
    }
  }

  pub fn to_slice(&self, a: &[u8]) -> request::Slice {
    let start = a.as_ptr() as usize - self.buffer.as_ptr() as usize;
    (start, start + a.len())
  }
}

impl<S, T> Future for Reader<S, T>
where
  T: router::FunctionCall,
  S: AsyncRead,
{
  type Item = (request::Request, response::Response);
  type Error = std::io::Error;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    loop {
      match std::mem::replace(&mut self.future_state, FutureProcessState::Empty) {
        FutureProcessState::Empty => {
          // should never be reached
          unreachable!()
        }
        FutureProcessState::Ready((mut req, res)) => {
          // simple http reader
          loop {
            match self.read_state {
              ReaderState::Headers => {
                let mut headers = [httparse::EMPTY_HEADER; 50];
                let mut r = httparse::Request::new(&mut headers);

                // dbg!(&self.buffer);
                let status = r.parse(&self.buffer).map_err(|e| {
                  std::io::Error::new(std::io::ErrorKind::Other, "Could not parse request")
                });

                match status? {
                  httparse::Status::Complete(amt) => {
                    let mut headers: Vec<(String, request::Slice)> =
                      Vec::with_capacity(r.headers.len());

                    for header in r.headers.iter() {
                      let header_name = header.name.to_lowercase();
                      if header_name == "transfer-encoding" {
                        self.read_state = ReaderState::Chunk;
                        // we need to check if it is actually chunked below thing is not right
                        // self.chunked = true
                      }

                      if header_name == "content-length" {
                        // this is incorrect way to assign body state
                        self.read_state = ReaderState::Body;
                        self.body_size = std::str::from_utf8(header.value)
                          .expect("Wrong value in header")
                          .parse::<usize>()
                          .expect("Could not parse usize");
                      }

                      headers.push((header_name, self.to_slice(header.value)));
                    }

                    // if let ReaderState::Chunk != self.read_state {
                    //   self.read_state = ReaderState::Body;
                    // }
                    // match self.read_state {
                    //   ReaderState::Headers => {
                    //     // ReaderState::Body;
                    //     // should be different to read till socket is closed
                    //   }
                    //   ReaderState::Body => {}
                    //   ReaderState::Chunk => {}
                    // }

                    // request is ready reset all values
                    req.headers = headers;
                    req.method = self.to_slice(r.method.unwrap().as_bytes());
                    req.version = r.version.unwrap();
                    req.request_raw = self.buffer.split_to(amt);
                    req.data = BytesMut::with_capacity(0);
                    req.is_last = false;

                    // emit data to the client
                    let fut = unsafe { (*self.router_raw_pointer).find((req, res)) };
                    let fut = fut.into_future();
                    self.future_state = FutureProcessState::Processing(fut);

                    // we need to break from first loop as it is completed with request ready :)
                    break;
                  }
                  httparse::Status::Partial => {
                    // continue reading as no enough headers available
                  }
                }
              }
              ReaderState::Chunk => {
                // do proper chunk parsing
                if self.buffer.len() > 0 {
                  match httparse::parse_chunk_size(&self.buffer) {
                    Ok(httparse::Status::Complete((start, length))) => {
                      // need to do optimization of the  chunks
                      let mut len = start as u64 + length + 2;
                      if length == 0 {
                        len = start as u64 + length;
                      }

                      // make sure that we have enough data to process
                      if self.buffer.len() as u64 >= len {
                        match std::mem::replace(&mut req.on_data, request::OnData::Empty) {
                          request::OnData::Function(f) => {
                            // this is very dangerous need proper optimization
                            self.buffer.advance(start);
                            let data = self.buffer.split_to(length as usize);

                            if self.buffer.len() == 7
                              && self.buffer == BytesMut::from("\r\n0\r\n\r\n")
                            {
                              self.buffer.advance(7);
                              self.read_state = ReaderState::Headers;
                              req.is_last = true;
                            } else {
                              self.buffer.advance(2);
                            }

                            // this is very slow
                            req.data.unsplit(data);

                            // if 0 length then it is last chunk
                            if length == 0 {
                              self.read_state = ReaderState::Headers;
                              req.is_last = true;
                            }

                            let fut = Box::new((f)((req, res)).and_then(|(mut req, res)| {
                              req.on_data = request::OnData::Function(f);
                              Ok((req, res))
                            }));

                            let fut = fut.into_future();
                            self.future_state = FutureProcessState::Processing(fut);
                            break;
                          }
                          request::OnData::Empty => {}
                        }
                      }
                    }
                    Ok(httparse::Status::Partial) => {}
                    Err(e) => {
                      return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Could not parse chunk",
                      ));
                    }
                  };
                }
              }
              ReaderState::Body => {
                if self.buffer.len() >= self.body_size {
                  let data = self.buffer.split_to(self.body_size);
                  self.read_state = ReaderState::Headers;
                  self.body_size = 0;

                  match std::mem::replace(&mut req.on_data, request::OnData::Empty) {
                    request::OnData::Function(f) => {
                      req.data = data;
                      req.is_last = true;

                      let fut = Box::new((f)((req, res)).and_then(|(mut req, res)| {
                        req.on_data = request::OnData::Function(f);
                        Ok((req, res))
                      }));

                      let fut = fut.into_future();
                      self.future_state = FutureProcessState::Processing(fut);
                      break;
                    }
                    request::OnData::Empty => {}
                  }
                }
              }
            }

            if !self.buffer.has_remaining_mut() {
              self.buffer.reserve(1024);
            }

            match self.socket.read_buf(&mut self.buffer)? {
              Async::Ready(0) => {
                return Ok(Async::Ready((req, res)));
              }
              Async::Ready(_) => {}
              Async::NotReady => {
                self.future_state = FutureProcessState::Ready((req, res));
                return Ok(Async::NotReady);
              }
            }
          }
        }
        FutureProcessState::Processing(mut fut) => match fut.poll()? {
          Async::Ready(v) => self.future_state = FutureProcessState::Ready(v),
          Async::NotReady => {
            self.future_state = FutureProcessState::Processing(fut);
            return Ok(Async::NotReady);
          }
        },
      }
    }
  }
}
