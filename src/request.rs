// need to add custom http request parser
use std::io;

use bytes::BytesMut;
use http::Uri;

type Slice = (usize, usize);

/// Contains http request information.
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

    //TODO: need to change this part to use slices
    let mut headers = Vec::with_capacity(r.headers.len());
    for header in r.headers.iter() {
      // this is not optimal (need to rethink whole connection)
      let value = unsafe { std::str::from_utf8_unchecked(header.value) }.to_string();
      headers.push((
        header.name.to_string(),
        // this part is very slow
        value,
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

  /// Get reference to Uri which contains all req path information.
  ///
  /// ```
  /// let uri_info = req.uri();
  /// let path = uri_info.path();
  /// // and so on
  /// ```
  pub fn uri(&self) -> &Uri {
    &self.uri
  }

  /// Get body of the request returns empty [u8] if no body provided.
  ///
  /// ```
  /// let req_body = req.body();
  /// ```
  pub fn body(&self) -> &[u8] {
    self.slice(&self.body)
  }

  /// Get request method.
  ///
  /// ```
  /// let method = req.method();
  /// ```
  pub fn method(&self) -> &str {
    std::str::from_utf8(self.slice(&self.method)).unwrap()
  }

  /// Get params from related to the Router.
  ///
  /// ```
  /// // if you have router with :
  /// router.get("/:hello", |req: Request| {
  ///   // you can get `hello` param from `params`
  ///   // make sure to check if value present
  ///   let params = req.params().unwrap();
  ///
  ///   for param in params {
  ///       // param.0 is `hello` and param.1 is value
  ///   }
  /// })
  /// ```
  pub fn params(&self) -> &Option<Vec<(&'static str, String)>> {
    &self.params
  }

  /// Get request http version (minor http version)
  ///
  /// ```
  /// // version will be 1 -> http 1.1 and 0 -> http 1.0
  /// let version = req.version();
  /// ```
  pub fn version(&self) -> u8 {
    self.version
  }

  /// Get http request headers
  ///
  /// ```
  /// let headers = req.headers();
  ///
  /// for header in headers {
  ///    // header.0 is header name and header.1 is value
  /// }
  /// ```
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
