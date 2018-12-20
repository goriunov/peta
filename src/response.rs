use crate::writer;

use std::fmt::{self, Write};

use tokio::net::TcpStream;

use bytes::{BufMut, BytesMut};

pub struct Response {
  headers: Vec<(String, String)>,
  response: Vec<u8>,
  status_message: StatusMessage,
}

pub enum StatusMessage {
  OK,
  NOT_FOUND,

  // custom status implementation
  Custom(u32, String),
}

impl Response {
  pub fn new() -> Response {
    Response {
      headers: Vec::new(),
      response: Vec::new(),
      status_message: StatusMessage::OK,
    }
  }

  pub fn status(mut self, status: StatusMessage) -> Response {
    self.status_message = status;
    self
  }

  pub fn custom_status(mut self, code: u32, message: &str) -> Response {
    self.status_message = StatusMessage::Custom(code, message.to_string());
    self
  }

  pub fn header(mut self, name: &str, val: &str) -> Response {
    self.headers.push((name.to_string(), val.to_string()));
    self
  }

  pub fn body(mut self, s: &str) -> Response {
    self.response = s.as_bytes().to_vec();
    self
  }

  pub fn write(
    &self,
    writer: tokio::io::WriteHalf<TcpStream>,
  ) -> writer::WriteAll<tokio::io::WriteHalf<TcpStream>> {
    let mut buf = BytesMut::with_capacity(4096);
    let length = self.response.len();

    write!(
      FastWrite(&mut buf),
      "\
       HTTP/1.1 {}\r\n\
       Server: Ultra\r\n\
       Content-Length: {}\r\n\
       ",
      self.status_message,
      length,
      // need to put date
    )
    .unwrap();

    for &(ref k, ref v) in &self.headers {
      push(&mut buf, k.as_bytes());
      push(&mut buf, ": ".as_bytes());
      push(&mut buf, v.as_bytes());
      push(&mut buf, "\r\n".as_bytes());
    }

    push(&mut buf, "\r\n".as_bytes());
    push(&mut buf, self.response.as_slice());

    writer::write_all(writer, buf)
  }
}

fn push(buf: &mut BytesMut, data: &[u8]) {
  buf.reserve(data.len());
  unsafe {
    buf.bytes_mut()[..data.len()].copy_from_slice(data);
    buf.advance_mut(data.len());
  }
}

struct FastWrite<'a>(&'a mut BytesMut);

impl<'a> fmt::Write for FastWrite<'a> {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    push(&mut *self.0, s.as_bytes());
    Ok(())
  }

  fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
    fmt::write(self, args)
  }
}

impl fmt::Display for StatusMessage {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      // need to add all list of responses
      StatusMessage::OK => f.pad("200 OK"),
      StatusMessage::NOT_FOUND => f.pad("404 Not Found"),
      StatusMessage::Custom(c, ref s) => write!(f, "{} {}", c, s),
    }
  }
}
