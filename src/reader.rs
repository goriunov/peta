use super::*;

enum ProcessState {
  Empty,
  Processing(ReturnFuture),
  Ready(ReqResTuple),
}

#[derive(PartialEq)]
enum ReadState {
  Request,
  Chunk,
  Body,
}

pub struct Reader<T> {
  req_func: OnData,
  socket: ReadHalf,
  buffer: BytesMut,
  body_size: usize,
  read_state: ReadState,
  router_raw: *const T,
  process_state: ProcessState,
}

impl<T> Reader<T>
where
  T: RouterSearch,
{
  pub fn new((socket, write_socket): (ReadHalf, WriteHalf), router: &T) -> Reader<T> {
    Reader {
      socket,
      req_func: OnData::Empty,
      buffer: BytesMut::with_capacity(1024),
      body_size: 0,
      router_raw: router as *const T,
      read_state: ReadState::Request,
      process_state: ProcessState::Ready((
        request::Request::new(),
        response::Response::new(write_socket),
      )),
    }
  }

  pub fn to_slice(&self, a: &[u8]) -> Slice {
    let start = a.as_ptr() as usize - self.buffer.as_ptr() as usize;
    (start, start + a.len())
  }
}

impl<T> Future for Reader<T>
where
  T: RouterSearch,
{
  type Item = ReqResTuple;
  type Error = std::io::Error;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    loop {
      match std::mem::replace(&mut self.process_state, ProcessState::Empty) {
        ProcessState::Empty => unreachable!(), // this should never be called
        ProcessState::Processing(mut fut) => {
          // poll future
          match fut.poll()? {
            Async::Ready((mut req, res)) => {
              // fetch function from request in to the reader for easier execution
              if req.has_on_data {
                req.has_on_data = false;
                self.req_func = std::mem::replace(&mut req.on_data, OnData::Empty);
              }

              self.process_state = ProcessState::Ready((req, res))
            }
            Async::NotReady => {
              self.process_state = ProcessState::Processing(fut);
              return Ok(Async::NotReady);
            }
          }
        }
        ProcessState::Ready((mut req, res)) => {
          // do main parse logic
          loop {
            match self.read_state {
              ReadState::Body => {
                if self.buffer.len() >= self.body_size {
                  let data = self.buffer.split_to(self.body_size);
                  self.read_state = ReadState::Request;
                  self.body_size = 0;
                  match &self.req_func {
                    OnData::Function(f) => {
                      req.data = data;
                      let fut = (f)((req, res));
                      let fut = fut.into_future();
                      self.process_state = ProcessState::Processing(fut);
                      break;
                    }
                    OnData::Empty => {}
                  }
                }
              }
              ReadState::Chunk => {
                // handle chunks
              }
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

                          // we need to handle body errors properly
                          self.body_size = std::str::from_utf8(header.value)
                            .expect("Wrong value in header")
                            .parse::<usize>()
                            .expect("Could not parse usize");
                        }
                      }

                      headers.push((header_name, self.to_slice(header.value)));
                    }

                    self.buffer.split_to(amt);
                    // everything is parsed we can process

                    let fut = unsafe { (*self.router_raw).find((req, res)) };
                    let fut = fut.into_future();
                    self.process_state = ProcessState::Processing(fut);
                    break;
                  }
                  Err(_e) => {
                    // we need to close socket and send error response to the client
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
              // We have some data need to check it in next iter
              Async::Ready(_) => {}
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
