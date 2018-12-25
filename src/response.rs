use super::writer;

use bytes::BytesMut;
use tokio::prelude::*;

pub struct Response {
  body: Vec<u8>,
  status: String,
  headers: Vec<(String, String)>,
}

impl Response {
  pub fn new() -> Response {
    Response {
      body: Vec::new(),
      status: String::new(),
      headers: Vec::with_capacity(50),
    }
  }

  pub fn header(&mut self, name: &str, value: &str) {}

  pub fn body_vec(&mut self, body: Vec<u8>) {
    self.body = body;
  }

  pub fn body_str(&mut self, body: &str) {
    self.body = body.as_bytes().to_vec();
  }

  // we should not pass buff in here
  pub fn write<S: AsyncWrite>(&self, writer: S, buff: BytesMut) -> writer::WriteAll<S> {
    writer::write_all(writer, buff)
  }
}
