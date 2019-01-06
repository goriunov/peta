use super::date;
use super::status;
use super::writer;

use bytes::{BufMut, BytesMut};
use tokio::prelude::*;

/// Creates HTTP response.
///
/// # Example
/// ```
/// let res = Response::new();
///
/// res.status(status::OK);
/// res.header("key", "value");
/// res.header("another", "value");
///
/// res.body_str("Hello world");
///
/// // write response to the socket
/// res.write(writer)
///
/// ```
pub struct Response {
  // consider to use different body structure
  body: Option<Vec<u8>>,
  status: Option<&'static str>,
  headers: Vec<(&'static str, &'static str)>,
}

impl Response {
  /// Create new instance of Response.
  ///
  /// ```
  /// let mut res = Response::new();
  /// ```
  pub fn new() -> Response {
    Response {
      body: None,
      status: None,
      headers: Vec::with_capacity(50),
    }
  }

  /// Attache status to the http response (can use `status` module).
  ///
  /// ```
  /// res.status(status::OK);
  /// ```
  pub fn status(&mut self, status: &'static str) {
    self.status = Some(status);
  }

  /// Attache single header to http response, can be reused.
  ///
  /// ```
  /// res.header("Header", "Value");
  /// ```
  pub fn header(&mut self, name: &'static str, value: &'static str) {
    self.headers.push((name, value));
  }

  // need to add different ways to add body
  /// Attache `Vec` body to the http response.
  ///
  /// ```
  /// let body = vec![....];
  /// res.body_vec(body);
  /// ```
  pub fn body_vec(&mut self, body: Vec<u8>) {
    self.body = Some(body);
  }

  /// Transform `&str` to `Vec` and attach it to the http response.
  ///
  /// ```
  /// res.body_str("Hello world");
  /// ```
  pub fn body_str(&mut self, body: &str) {
    self.body = Some(body.as_bytes().to_vec());
  }

  // we should not pass buff in here
  /// Process Response and return future which writes response to the `AsyncWrite` socket.
  ///
  /// ```
  /// let (read, write) = socket.split();
  /// ...
  /// res.write(write);
  /// ```
  pub fn write<S: AsyncWrite>(&self, writer: S) -> writer::WriteAll<S> {
    // write all data together
    let mut buf = BytesMut::with_capacity(4096);

    // write status, set default to 200 if does not exist
    push(&mut buf, b"HTTP/1.1 ");
    push(&mut buf, self.status.unwrap_or(status::OK).as_bytes());
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
    match &self.body {
      Some(body) => {
        push(&mut buf, b"content-length: ");
        push(&mut buf, body.len().to_string().as_bytes());
        push(&mut buf, b"\r\n\r\n");
        push(&mut buf, body.as_slice());
      }
      None => push(&mut buf, b"content-length: 0\r\n\r\n"),
    };

    // write to socket
    writer::write_all(writer, buf)
  }
}

pub(crate) fn push(buf: &mut BytesMut, data: &[u8]) {
  if buf.remaining_mut() < data.len() {
    buf.reserve(data.len());
  }

  unsafe {
    buf.bytes_mut()[..data.len()].copy_from_slice(data);
    buf.advance_mut(data.len());
  }
}
