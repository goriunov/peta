// consider to add custom http parser
use std::io;

use bytes::BytesMut;
use http::Uri;

type Slice = (usize, usize);

// need to add headers
pub struct Request {
  data: BytesMut,
  uri: Uri,
  body: Slice,
  method: Slice,
  version: u8,
  // optimize params
  params: Option<Vec<(&'static str, String)>>,
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
      uri: r.path.unwrap().parse::<Uri>().unwrap(),
      method: to_slice(r.method.unwrap().as_bytes()),
      version: r.version.unwrap(),
      body: (amt, buffer.len()),
      params: None,
      // move buff
      data: buffer.take(),
    }))
  }

  pub fn uri(&self) -> &Uri {
    &self.uri
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

  pub fn params(&self) -> &Option<Vec<(&'static str, String)>> {
    &self.params
  }

  pub fn set_params(&mut self, params: Option<Vec<(&'static str, String)>>) {
    self.params = params;
  }

  fn slice(&self, slice: &Slice) -> &[u8] {
    &self.data[slice.0..slice.1]
  }
}
