use bytes::{BufMut, BytesMut};
use tokio::prelude::*;

use super::date;
use super::writer;

pub enum ResponseState {
  Chunk,
  Completed,
}

pub struct Response {
  pub chunked: bool,
  pub completed: bool,
  body: Vec<u8>,
  status: &'static str,
  headers: Vec<(&'static str, &'static str)>,
}

impl Response {
  pub fn new() -> Response {
    Response {
      chunked: false,
      completed: false,
      body: Vec::with_capacity(0),
      status: "",
      headers: Vec::with_capacity(50),
    }
  }

  pub fn is_completed(&self) -> bool {
    self.completed
  }

  pub fn status(&mut self, status: &'static str) {
    self.status = status;
  }

  pub fn header(&mut self, name: &'static str, value: &'static str) {
    self.headers.push((name, value));
  }

  pub fn body_vec(&mut self, body: Vec<u8>) {
    self.body = body;
  }

  pub fn body_str(&mut self, body: &str) {
    self.body = body.as_bytes().to_vec();
  }

  pub fn write<S: AsyncWrite>(&mut self, writer: S) -> writer::WriteAll<S> {
    // write all data together
    let mut buf = BytesMut::with_capacity(4096);

    // need to set default response code
    // write status
    push(&mut buf, b"HTTP/1.1 ");
    push(&mut buf, self.status.as_bytes());
    push(&mut buf, b"\r\n");

    // write headers
    for val in &self.headers {
      push(&mut buf, val.0.as_bytes());
      push(&mut buf, b": ");
      push(&mut buf, val.1.as_bytes());
      push(&mut buf, b"\r\n");
    }

    // set date header
    date::set_date_header(&mut buf);

    // add content-length and actual body
    let body_len = self.body.len();

    if body_len > 0 {
      push(&mut buf, b"content-length: ");
      push(&mut buf, body_len.to_string().as_bytes());
      push(&mut buf, b"\r\n\r\n");
      push(&mut buf, self.body.as_slice());
    } else {
      push(&mut buf, b"content-length: 0\r\n\r\n")
    }

    // write to socket
    self.completed = true;
    writer::write_all(writer, buf)
  }

  // will be used to write chunks
  pub fn write_chunk<S: AsyncWrite>(&mut self, writer: S, is_last: bool) -> writer::WriteAll<S> {
    let mut buf = BytesMut::with_capacity(4096);

    if !self.chunked {
      self.chunked = true;

      push(&mut buf, b"HTTP/1.1 ");
      push(&mut buf, self.status.as_bytes());
      push(&mut buf, b"\r\n");

      // write headers
      for val in &self.headers {
        push(&mut buf, val.0.as_bytes());
        push(&mut buf, b": ");
        push(&mut buf, val.1.as_bytes());
        push(&mut buf, b"\r\n");
      }
      // set date header
      date::set_date_header(&mut buf);

      push(&mut buf, b"transfer-encoding: chunked\r\n\r\n");
    }

    let body_len = self.body.len();
    push(&mut buf, format!("{:x}", body_len).as_bytes());
    push(&mut buf, b"\r\n");
    push(&mut buf, self.body.as_slice());
    push(&mut buf, b"\r\n");

    if is_last {
      push(&mut buf, b"0\r\n\r\n");
      self.chunked = false;
      self.completed = true;
    }

    dbg!(&buf);

    writer::write_all(writer, buf)
  }
}

// fast unsafe push
pub(crate) fn push(buf: &mut BytesMut, data: &[u8]) {
  if buf.remaining_mut() < data.len() {
    buf.reserve(data.len());
  }

  unsafe {
    buf.bytes_mut()[..data.len()].copy_from_slice(data);
    buf.advance_mut(data.len());
  }
}
