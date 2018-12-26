use super::writer;

use bytes::{BufMut, BytesMut};
use tokio::prelude::*;

pub struct Response {
  body: Option<Vec<u8>>,
  status: String,
  headers: Vec<String>,
}

impl Response {
  pub fn new() -> Response {
    Response {
      body: None,
      status: String::with_capacity(100),
      headers: Vec::with_capacity(50),
    }
  }

  // add enum with most common statuses
  pub fn status(&mut self, status: &str) {
    self.status.push_str("HTTP/1.1 ");
    self.status.push_str(status);
    self.status.push_str("\r\n");
  }

  pub fn header(&mut self, name: &str, value: &str) {
    let mut header = String::with_capacity(4 + name.len() + value.len());
    header.push_str(name);
    header.push_str(": ");
    header.push_str(value);
    header.push_str("\r\n");
    self.headers.push(header);
    // add header to the header array
  }

  pub fn body_vec(&mut self, body: Vec<u8>) {
    self.body = Some(body);
  }

  pub fn body_str(&mut self, body: &str) {
    self.body = Some(body.as_bytes().to_vec());
  }

  // we should not pass buff in here
  pub fn write<S: AsyncWrite>(&self, writer: S) -> writer::WriteAll<S> {
    // write all data together
    let mut buf = BytesMut::with_capacity(4096);

    push(&mut buf, self.status.as_bytes());

    // loop and add headers
    for &ref val in &self.headers {
      push(&mut buf, val.as_bytes());
    }

    // add content length and
    let mut content_headers = String::with_capacity(50);

    match &self.body {
      Some(body) => {
        content_headers.push_str("Server: Peta\r\nContent-Length: ");
        content_headers.push_str(&body.len().to_string());
        push(&mut buf, content_headers.as_bytes());
        push(&mut buf, b"\r\n\r\n");
        push(&mut buf, body.as_slice());
      }
      None => {
        content_headers.push_str("Server: Peta\r\nContent-Length: 0");
        push(&mut buf, content_headers.as_bytes());
        push(&mut buf, b"\r\n\r\n");
      }
    }

    writer::write_all(writer, buf)
  }
}

fn push(buf: &mut BytesMut, data: &[u8]) {
  if buf.remaining_mut() < data.len() {
    buf.reserve(data.len());
  }

  unsafe {
    buf.bytes_mut()[..data.len()].copy_from_slice(data);
    buf.advance_mut(data.len());
  }
}
