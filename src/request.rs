use bytes::BytesMut;

type Slice = (usize, usize);

pub struct Request {
  data: BytesMut,
  body: BytesMut,
  path: Slice,
  method: Slice,
  version: u8,
  headers: Vec<(Slice, Slice)>,
}

impl Request {
  pub fn decode(buf: &mut BytesMut) -> Result<Option<Request>, ()> {
    let (method, path, version, headers, amt) = {
      let mut headers = [httparse::EMPTY_HEADER; 16];
      let mut r = httparse::Request::new(&mut headers);

      let status = r.parse(buf).expect("Could not parse http");

      let amt = match status {
        httparse::Status::Complete(amt) => amt,
        httparse::Status::Partial => return Ok(None),
      };

      let to_slice = |a: &[u8]| {
        let start = a.as_ptr() as usize - buf.as_ptr() as usize;
        assert!(start < buf.len());
        (start, start + a.len())
      };

      (
        to_slice(r.method.unwrap().as_bytes()),
        to_slice(r.path.unwrap().as_bytes()),
        r.version.unwrap(),
        r.headers
          .iter()
          .map(|h| (to_slice(h.name.as_bytes()), to_slice(h.value)))
          .collect(),
        amt,
      )
    };

    Ok(
      Request {
        method: method,
        path: path,
        version: version,
        headers: headers,
        data: buf.split_to(amt),
        body: buf.split_off(0),
      }
      .into(),
    )
  }

  // need to add reader for headers
  pub fn headers(&self) -> Headers {
    Headers {
      req: self,
      headers: self.headers.iter(),
    }
  }

  pub fn method(&self) -> &str {
    std::str::from_utf8(self.slice(&self.method)).unwrap()
  }

  pub fn path(&self) -> &str {
    std::str::from_utf8(self.slice(&self.path)).unwrap()
  }

  pub fn version(&self) -> u8 {
    self.version
  }

  pub fn body(&self) -> &BytesMut {
    &self.body
  }

  pub fn body_mut(&mut self) -> &mut BytesMut {
    &mut self.body
  }

  fn slice(&self, slice: &Slice) -> &[u8] {
    &self.data[slice.0..slice.1]
  }
}

// Header iterator :)
pub struct Headers<'req> {
  headers: std::slice::Iter<'req, (Slice, Slice)>,
  req: &'req Request,
}

impl<'req> Iterator for Headers<'req> {
  type Item = (&'req str, &'req [u8]);

  fn next(&mut self) -> Option<(&'req str, &'req [u8])> {
    self.headers.next().map(|&(ref a, ref b)| {
      let a = self.req.slice(a);
      let b = self.req.slice(b);
      (std::str::from_utf8(a).unwrap(), b)
    })
  }
}
