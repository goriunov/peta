use bytes::{BufMut, BytesMut};

use super::date;
use super::writer;

type Writer = tokio::io::WriteHalf<tokio::net::TcpStream>;

enum WriteState {
  Waiting,
  Writing,
}

pub struct Response {
  pub(crate) writer: Writer,
  state: WriteState,
  status: &'static str,
  headers: Vec<(&'static str, &'static str)>,
}

impl Response {
  pub fn new(writer: Writer) -> Response {
    Response {
      writer,
      state: WriteState::Waiting,
      status: "",
      headers: Vec::with_capacity(50),
    }
  }

  pub fn status(&mut self, status: &'static str) {
    self.status = status;
  }

  pub fn header(&mut self, name: &'static str, value: &'static str) {
    self.headers.push((name, value));
  }

  pub fn write(mut self, body: &[u8]) -> writer::WriteAll {
    let mut buf = BytesMut::with_capacity(4096);

    match &self.state {
      WriteState::Waiting => {
        // prepare headers
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
        self.state = WriteState::Writing;
      }
      WriteState::Writing => {
        // ignore for now
      }
    };

    // we know that there is always data
    let body_len = body.len();
    push(&mut buf, format!("{:x}", body_len).as_bytes());
    push(&mut buf, b"\r\n");
    push(&mut buf, body);
    push(&mut buf, b"\r\n");

    writer::write_all(self, buf)
  }

  pub fn end(mut self, body: &[u8]) -> writer::WriteAll {
    let mut buf = BytesMut::with_capacity(4096);

    match &self.state {
      WriteState::Waiting => {
        // prepare headers
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
      WriteState::Writing => {
        self.state = WriteState::Waiting;

        // ignore for now
      }
    };

    let body_len = body.len();

    if body_len > 0 {
      push(&mut buf, format!("{:x}", body_len).as_bytes());
      push(&mut buf, b"\r\n");
      push(&mut buf, body);
      push(&mut buf, b"\r\n0\r\n\r\n");
    } else {
      push(&mut buf, b"0\r\n\r\n");
    }

    // dbg!(&buf);
    writer::write_all(self, buf)
  }

  pub fn write_h_1_0(self, body: &[u8]) -> writer::WriteAll {
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
    let body_len = body.len();

    if body_len > 0 {
      push(&mut buf, b"content-length: ");
      push(&mut buf, body_len.to_string().as_bytes());
      push(&mut buf, b"\r\n\r\n");
      push(&mut buf, body);
    } else {
      push(&mut buf, b"content-length: 0\r\n\r\n")
    }

    // write to socket
    writer::write_all(self, buf)
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
