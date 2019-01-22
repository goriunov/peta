use std::io;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::prelude::*;

use bytes::{BufMut, BytesMut};

use crate::request;

pub enum State {
  Body,
  Request,
  Chunk,
}

pub struct Reader<S> {
  state: State,
  req: request::Request,
  socket: S,
  buffer: BytesMut,
}

impl<S: AsyncRead> Reader<S> {
  pub fn new(socket: S) -> Reader<S> {
    let req = request::Request::new();

    Reader {
      req,
      socket,
      state: State::Request,
      buffer: BytesMut::with_capacity(1024),
    }
  }

  pub fn to_slice(&self, a: &[u8]) -> request::Slice {
    let start = a.as_ptr() as usize - self.buffer.as_ptr() as usize;
    (start, start + a.len())
  }
}

impl<S: AsyncRead> Stream for Reader<S> {
  type Item = (BytesMut, State);
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    // write reader
    loop {
      // check this only if we have some data in the buffer
      match self.state {
        State::Chunk => {
          // check if there is chunks available
          // find last chunk
        }
        State::Body => {
          // check if we have got whole body
          self.state = State::Request;
          return Ok(Async::Ready(Some((self.buffer.take(), State::Body))));
        }
        _ => {}
      };

      // continue reading
      if !self.buffer.has_remaining_mut() {
        self.buffer.reserve(1024);
      }

      match self.socket.read_buf(&mut self.buffer) {
        Ok(Async::Ready(0)) => {
          // end connection
          return Ok(Async::Ready(None));
        }
        Ok(Async::Ready(_)) => {
          match self.state {
            State::Chunk => {
              // read chunk and try to stream it out
              //
            }
            State::Request => {
              let mut headers = [httparse::EMPTY_HEADER; 50];
              let mut r = httparse::Request::new(&mut headers);

              let status = r
                .parse(&self.buffer)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, "Could not parse request"));

              let amt = match status {
                Ok(httparse::Status::Complete(amt)) => amt,
                Ok(httparse::Status::Partial) => continue,
                Err(e) => {
                  // handle error properly
                  return Err(e);
                }
              };

              // find out which state it it !
              self.state = State::Body;

              return Ok(Async::Ready(Some((
                self.buffer.split_to(amt),
                State::Request,
              ))));
            }
            _ => continue,
          }
        }
        Ok(Async::NotReady) => {
          // always return not ready
          return Ok(Async::NotReady);
        }
        Err(e) => {
          // something fucked up
          return Err(e);
        }
      }
    }
    // loop {
    //   // extend buffer if needed
    //   if !self.buffer.has_remaining_mut() {
    //     self.buffer.reserve(1024);
    //   }

    //   match self.socket.read_buf(&mut self.buffer) {
    //     Ok(Async::Ready(0)) => {
    //       // end connection
    //       return Ok(Async::Ready(None));
    //     }
    //     Ok(Async::Ready(_)) => {
    //       continue;
    //     }
    //     Ok(Async::NotReady) => {
    //       let buff_len = self.buffer.len();

    //       println!("Notified called");

    //       if buff_len == 0 {
    //         return Ok(Async::NotReady);
    //       }

    //       // if request is chunked
    //       if self.req.is_chunked {

    //         // complete chunked parse implementation
    //       }

    //       // waiting for full body
    //       if self.req.is_waiting {
    //         // if it is still less then body size then wait
    //         if buff_len - self.req.amt < self.req.body_size {
    //           return Ok(Async::NotReady);
    //         }

    //         // body is ready to be processed
    //         // we know that `content` exists as it was created at the first iter
    //         self.req.content.as_mut().unwrap().data =
    //           Some(self.buffer.split_to(self.req.amt + self.req.body_size));

    //         // we can emit prepared request
    //         return Ok(Async::Ready(Some(std::mem::replace(
    //           &mut self.req,
    //           request::Request::new(),
    //         ))));
    //       }

    //       // set header for 50 max (may be need to increase it later)
    //       let mut headers = [httparse::EMPTY_HEADER; 50];
    //       let mut r = httparse::Request::new(&mut headers);

    //       let status = r.parse(&self.buffer).map_err(|e| {
    //         // we were not able to parse request we need to close connection
    //         // as request is incorrect
    //         io::Error::new(io::ErrorKind::Other, "Could not parse request")
    //       });

    //       self.req.amt = match status {
    //         Ok(httparse::Status::Complete(amt)) => amt,
    //         Ok(httparse::Status::Partial) => return Ok(Async::NotReady),
    //         Err(e) => {
    //           // handle error properly
    //           return Err(e);
    //         }
    //       };

    //       let mut headers: Vec<(String, request::Slice)> = Vec::with_capacity(r.headers.len());

    //       // loop through headers and find out everything about them
    //       for header in r.headers.iter() {
    //         // parse headers
    //         let header_name = header.name.to_lowercase();

    //         // take transfer-coding: x, chunk as priority
    //         if header_name == "transfer-coding" {
    //           // check if it is actual chunked encoding
    //           let value_len = header.value.len();
    //           if value_len >= 7 && &header.value[value_len - 7..] == b"chunked" {
    //             // this request is chunked
    //             self.req.is_chunked = true;

    //             // we have some part of body
    //             if self.req.amt < buff_len {

    //               // if parse completed then
    //               // it is not chunked any more

    //               // if not completed then parse as much as we can and create body
    //             }
    //           }
    //         }

    //         if header_name == "content-length" {
    //           self.req.body_size = std::str::from_utf8(header.value)
    //             .expect("Wrong value in header")
    //             .parse::<usize>()
    //             .expect("Could not parse usize");

    //           // add error handling for 400 Bad request if we have 2 content-length or invalid value

    //           // if current buff body is less then we need then wait
    //           if buff_len - self.req.amt < self.req.body_size {
    //             self.req.is_waiting = true;
    //           }
    //         }

    //         headers.push((header_name, self.to_slice(header.value)));
    //       }

    //       // need to find out how do we handle chunked data creation
    //       self.req.content = Some(request::Content {
    //         headers,
    //         data: None,
    //         body: (self.req.amt, self.req.amt + self.req.body_size),
    //         method: self.to_slice(r.method.unwrap().as_bytes()),
    //       });

    //       if !self.req.is_waiting && !self.req.is_chunked {
    //         self.req.content.as_mut().unwrap().data =
    //           Some(self.buffer.split_to(self.req.amt + self.req.body_size));

    //         return Ok(Async::Ready(Some(std::mem::replace(
    //           &mut self.req,
    //           request::Request::new(),
    //         ))));
    //       }

    //       return Ok(Async::NotReady);
    //     }
    //     Err(e) => {
    //       // something fucked up
    //       return Err(e);
    //     }
    //   };
    // }
  }
}

// pub struct SocketReader {
//   socket: tokio::io::ReadHalf<tokio::net::TcpStream>,
//   buffer: BytesMut,
//   httpReqTask: Option<Arc<Mutex<futures::task::Task>>>,
//   bodyTask: Option<Arc<Mutex<futures::task::Task>>>,
// }

// impl SocketReader {
//   pub fn new(socket: tokio::io::ReadHalf<tokio::net::TcpStream>) -> SocketReader {
//     SocketReader {
//       socket,
//       buffer: BytesMut::with_capacity(1024),
//     }
//   }
// }

// impl Stream for SocketReader {
//   type Item = ();
//   type Error = io::Error;

//   fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
//     loop {}
//   }
// }
