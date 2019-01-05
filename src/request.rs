use std::io;

use bytes::BytesMut;
use http::Uri;

type Slice = (usize, usize);

// need optimization !!! sooo slow this part

// need to add headers
pub struct Request {
  data: BytesMut,
  uri: Uri,
  body: Slice,
  method: Slice,
  version: u8,
  headers: Vec<(String, String)>,
  params: Option<Vec<(&'static str, String)>>,
}

impl Request {
  pub(crate) fn parse(buffer: &mut BytesMut) -> Result<Option<Request>, io::Error> {
    // consider to replace current header parser to own implementation
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

    let mut headers = Vec::with_capacity(r.headers.len());
    for header in r.headers.iter() {
      // this is not optimal (need to rethink whole connection)
      headers.push((
        header.name.to_string(),
        String::from_utf8(header.value.to_vec()).unwrap(),
      ));
    }

    Ok(Some(Request {
      uri: r.path.unwrap().parse::<Uri>().unwrap(),
      method: to_slice(r.method.unwrap().as_bytes()),
      version: r.version.unwrap(),
      body: (amt, buffer.len()),
      params: None,
      headers: headers,
      // move buff in
      data: buffer.take(),
    }))
  }

  pub fn uri(&self) -> &Uri {
    &self.uri
  }

  pub fn body(&self) -> &[u8] {
    self.slice(&self.body)
  }

  pub fn params(&self) -> &Option<Vec<(&'static str, String)>> {
    &self.params
  }

  pub fn method(&self) -> &str {
    std::str::from_utf8(self.slice(&self.method)).unwrap()
  }

  pub fn version(&self) -> u8 {
    self.version
  }

  pub fn headers(&self) -> &Vec<(String, String)> {
    &self.headers
  }

  pub(crate) fn set_params(&mut self, params: Option<Vec<(&'static str, String)>>) {
    self.params = params;
  }

  fn slice(&self, slice: &Slice) -> &[u8] {
    &self.data[slice.0..slice.1]
  }
}
