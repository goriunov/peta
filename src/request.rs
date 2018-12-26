// consider to add custom http parser
use std::io;

use bytes::BytesMut;

type Slice = (usize, usize);

// need to add headers
pub struct Request {
  body: Slice,
  path: Slice,
  method: Slice,
  data: BytesMut,
  version: u8,
}

impl Request {
  pub fn parse(buffer: &mut BytesMut) -> Result<Option<Request>, io::Error> {
    // we need to handle headers properly currently default 50
    let mut headers = [httparse::EMPTY_HEADER; 50];
    let mut r = httparse::Request::new(&mut headers);

    let status = r.parse(buffer).map_err(|e| {
      let msg = format!("Failed to parse http request: {:?}", e);
      io::Error::new(io::ErrorKind::Other, msg)
    });

    let amt = match status {
      Ok(httparse::Status::Complete(amt)) => amt,
      Ok(httparse::Status::Partial) => return Ok(None),
      Err(e) => return Err(e),
    };

    let to_slice = |a: &[u8]| {
      let start = a.as_ptr() as usize - buffer.as_ptr() as usize;
      (start, start + a.len())
    };

    Ok(Some(Request {
      path: to_slice(r.method.unwrap().as_bytes()),
      method: to_slice(r.path.unwrap().as_bytes()),
      version: r.version.unwrap(),
      body: (amt, buffer.len()),
      // move buff
      data: buffer.take(),
    }))
  }

  pub fn path(&self) -> &str {
    std::str::from_utf8(self.slice(&self.path)).unwrap()
  }

  pub fn method(&self) -> &str {
    std::str::from_utf8(self.slice(&self.method)).unwrap()
  }

  pub fn body(&self) -> &[u8] {
    self.slice(&self.body)
  }

  pub fn version(&self) -> u8 {
    self.version
  }

  fn slice(&self, slice: &Slice) -> &[u8] {
    &self.data[slice.0..slice.1]
  }
}
