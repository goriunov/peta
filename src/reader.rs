use super::*;

enum ProcessState {
  Empty,
  Processing(ReturnFuture),
  Ready((request::Request, response::Response)),
}

#[derive(Debug, PartialEq)]
enum ReadState {
  Request,
  Chunk,
  Body,
}

pub struct Reader<S> {
  socket: S,
  buffer: BytesMut,
  read_state: ReadState,
  process_state: ProcessState,
}

impl<S: AsyncRead> Reader<S> {
  pub fn new(socket: S) -> Reader<S> {
    Reader {
      socket,
      buffer: BytesMut::with_capacity(1024),
      read_state: ReadState::Request,
      process_state: ProcessState::Ready((request::Request::new(), response::Response::new())),
    }
  }

  pub fn to_slice(&self, a: &[u8]) -> Slice {
    let start = a.as_ptr() as usize - self.buffer.as_ptr() as usize;
    (start, start + a.len())
  }
}

impl<S: AsyncRead> Future for Reader<S> {
  type Item = (request::Request, response::Response);
  type Error = std::io::Error;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    loop {
      match std::mem::replace(&mut self.process_state, ProcessState::Empty) {
        ProcessState::Empty => unreachable!(), // this should never be called
        ProcessState::Processing(mut fut) => {
          // poll future
          match fut.poll()? {
            Async::Ready(v) => self.process_state = ProcessState::Ready(v),
            Async::NotReady => {
              self.process_state = ProcessState::Processing(fut);
              return Ok(Async::NotReady);
            }
          }
        }
        ProcessState::Ready((req, res)) => {
          // do main parse logic
          loop {
            match self.read_state {
              ReadState::Body => {}
              ReadState::Chunk => {}
              ReadState::Request => {
                let mut headers = [httparse::EMPTY_HEADER; 50];
                let mut r = httparse::Request::new(&mut headers);

                // parse available data
                match r.parse(&self.buffer) {
                  Ok(httparse::Status::Partial) => {} // continue reading (not enough data)
                  Ok(httparse::Status::Complete(amt)) => {
                    let mut headers: Vec<(String, Slice)> = Vec::with_capacity(r.headers.len());

                    for header in r.headers.iter() {
                      let header_name = header.name.to_lowercase();

                      if self.read_state != ReadState::Chunk {
                        if header_name == "transfer-encoding" {
                          // we may have got chunk
                          // check if we actually have chunk
                          self.read_state = ReadState::Chunk;
                        } else if header_name == "content-length" {
                          self.read_state = ReadState::Body;
                          // we have got content length
                          // try and pars string
                        }
                      }

                      headers.push((header_name, self.to_slice(header.value)));
                    }
                  }
                  Err(e) => {
                    // we probably need to close socket and send error response to the client
                    return Err(std::io::Error::new(
                      std::io::ErrorKind::Other,
                      "Could not parse request",
                    ));
                  }
                }
              }
            }

            if !self.buffer.has_remaining_mut() {
              self.buffer.reserve(1024);
            }

            match self.socket.read_buf(&mut self.buffer)? {
              // 0 socket is closed :)
              Async::Ready(0) => return Ok(Async::Ready((req, res))),
              Async::Ready(_) => {
                // continue reading data
              }
              Async::NotReady => {
                // nothing has been read set our state to ready to process new data in next wake up
                self.process_state = ProcessState::Ready((req, res));
                return Ok(Async::NotReady);
              }
            }
          }
        }
      }
    }
  }
}
