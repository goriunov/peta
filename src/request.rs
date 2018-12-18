use bytes::BytesMut;

type Slice = (usize, usize);

pub struct Request {
  method: Slice,
  path: Slice,
  version: u8,
  // TODO: use a small vec to avoid this unconditional allocation
  headers: Vec<(Slice, Slice)>,
  data: BytesMut,
}

impl Request {
  pub fn decode(buf: &mut BytesMut) -> Result<Option<Request>, ()> {
    // parse http
    let (method, path, version, headers, amt) = {
      let mut headers = [httparse::EMPTY_HEADER; 16];
      let mut r = httparse::Request::new(&mut headers);

      let status = r.parse(buf).expect("Could not parse http");

      let amt = match status {
        httparse::Status::Complete(amt) => amt,
        httparse::Status::Partial => return Ok(None),
      };

      let toslice = |a: &[u8]| {
        let start = a.as_ptr() as usize - buf.as_ptr() as usize;
        assert!(start < buf.len());
        (start, start + a.len())
      };

      (
        toslice(r.method.unwrap().as_bytes()),
        toslice(r.path.unwrap().as_bytes()),
        r.version.unwrap(),
        r.headers
          .iter()
          .map(|h| (toslice(h.name.as_bytes()), toslice(h.value)))
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
      }
      .into(),
    )
  }

  // need to add reader for headers

  pub fn method(&self) -> &str {
    std::str::from_utf8(self.slice(&self.method)).unwrap()
  }

  pub fn path(&self) -> &str {
    std::str::from_utf8(self.slice(&self.path)).unwrap()
  }

  pub fn version(&self) -> u8 {
    self.version
  }

  fn slice(&self, slice: &Slice) -> &[u8] {
    &self.data[slice.0..slice.1]
  }
}
