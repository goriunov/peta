use super::*;

#[derive(PartialEq)]
enum ReadState {
  Body,
  Chunk,
  Request,
}

enum ProcessState {
  Empty,
  Ready(ReqResTuple),
  Processing(ReturnFuture),
}

pub struct Reader<T> {
  socket: ReadHalf,
  buffer: BytesMut,
  req_func: OnData,
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
      buffer: BytesMut::with_capacity(1024),
      req_func: OnData::Empty,
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
          match fut.poll()? {
            Async::Ready((mut req, res)) => {
              // fetch function from request in to the reader for easier execution
              if req.has_data_function {
                req.has_data_function = false;
                self.req_func = std::mem::replace(&mut req.on_data, OnData::Empty);
              }

              self.process_state = ProcessState::Ready((req, res));
            }
            Async::NotReady => {
              self.process_state = ProcessState::Processing(fut);
              return Ok(Async::NotReady);
            }
          }
        }
        ProcessState::Ready((mut req, res)) => {
          loop {
            // check what reading state we are in
            match self.read_state {
              ReadState::Body => {
                if self.buffer.len() >= self.body_size {
                  // reset state and emit all data
                  let data = self.buffer.split_to(self.body_size);
                  self.body_size = 0;
                  self.read_state = ReadState::Request;

                  match &self.req_func {
                    OnData::Function(f) => {
                      req.data = data;
                      let fut = (f)((req, res));
                      self.process_state = ProcessState::Processing(fut.into_future());
                      break;
                    }
                    OnData::Empty => {} // process
                  }
                }
              }
              ReadState::Chunk => {
                if self.buffer.len() > 0 {
                  //TODO: handle chunks
                  match chunk::Chunk::parse(&mut self.buffer)? {
                    chunk::Status::Chunk(data) => {
                      match &self.req_func {
                        OnData::Function(f) => {
                          req.data.unsplit(data);
                          let fut = (f)((req, res));
                          self.process_state = ProcessState::Processing(fut.into_future());
                          break;
                        }
                        OnData::Empty => {} // process
                      }
                    }
                    chunk::Status::Last => {
                      // end this implementation
                      self.read_state = ReadState::Request;
                    }
                    chunk::Status::NotEnoughData => {}
                  };
                }
              }
              ReadState::Request => {
                let mut headers = [httparse::EMPTY_HEADER; 50];
                let mut r = httparse::Request::new(&mut headers);

                // parse available data
                match r.parse(&self.buffer) {
                  Ok(httparse::Status::Partial) => {} // continue reading (not enough data)
                  Ok(httparse::Status::Complete(amt)) => {
                    req.reset_headers(r.headers.len());

                    // always assume that we have data (even if there is no data)
                    self.read_state = ReadState::Body;

                    for header in r.headers.iter() {
                      // make all header's names the same case
                      let header_name = header.name.to_lowercase();

                      if self.read_state != ReadState::Chunk {
                        if header_name == "transfer-encoding" {
                          // TODO: check if we actually have chunk encoding
                          self.read_state = ReadState::Chunk;
                        } else if header_name == "content-length" {
                          //TODO: need to handle errors properly
                          self.body_size = std::str::from_utf8(header.value)
                            .expect("Wrong value in header")
                            .parse::<usize>()
                            .expect("Could not parse usize");
                        }
                      }

                      let mut buf = Vec::with_capacity(header.value.len());
                      unsafe {
                        // we can do unsafe copy
                        buf.bytes_mut()[..header.value.len()].copy_from_slice(header.value)
                      };
                      req.add_header(header_name, buf);
                    }

                    // empty previous function
                    self.req_func = OnData::Empty;

                    let method = self.to_slice(r.method.unwrap().as_bytes());
                    let version = r.version.unwrap();
                    req.init(version, method, self.buffer.split_to(amt));

                    let fut = unsafe { (*self.router_raw).find((req, res)) };
                    self.process_state = ProcessState::Processing(fut.into_future());
                    break;
                  }
                  Err(_e) => {
                    //TODO: need to close socket and send error response to the client
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
