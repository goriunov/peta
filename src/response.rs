use super::*;

pub struct Response {
  pub(crate) socket: WriteHalf,
  status: &'static str,
}

impl Response {
  pub fn new(socket: WriteHalf) -> Response {
    Response {
      socket,
      status: "200 Ok",
    }
  }

  pub fn status(&mut self, status: &'static str) {
    self.status = status;
  }

  pub fn write(self, body: &[u8]) -> writer::WriteAll {
    let mut buf = BytesMut::with_capacity(4096);

    push(&mut buf, b"HTTP/1.1 ");
    push(&mut buf, self.status.as_bytes());
    push(&mut buf, b"\r\n");

    date::set_date_header(&mut buf);

    let body_len = body.len();

    if body_len > 0 {
      push(&mut buf, b"content-length: ");
      push(&mut buf, body_len.to_string().as_bytes());
      push(&mut buf, b"\r\n\r\n");
      push(&mut buf, body);
    } else {
      push(&mut buf, b"content-length: 0\r\n\r\n")
    }

    writer::write_all(self, buf)
  }

  pub(crate) fn shutdown(&mut self) {
    // TODO: handle unwrap
    self.socket.shutdown().unwrap();
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
