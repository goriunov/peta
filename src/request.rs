use super::*;

pub struct Request {
  pub data: BytesMut,
  pub(crate) on_data: OnData,
  pub(crate) is_last: bool,
  pub(crate) has_function: bool,
  pub(crate) uri: Uri,
  method: String,
  version: u8,
  request_data: BytesMut,
  headers: hashbrown::HashMap<String, Vec<u8>>,
}

impl Request {
  pub fn new() -> Request {
    Request {
      has_function: false,
      is_last: false,
      version: 0,
      uri: Uri::default(),
      on_data: OnData::Empty,
      method: String::new(),
      headers: hashbrown::HashMap::new(),
      request_data: BytesMut::new(),
      data: BytesMut::new(),
    }
  }

  pub fn on_data<F>(&mut self, func: F)
  where
    F: Fn(ReqResTuple) -> ReturnFuture + Send + Sync + 'static,
  {
    self.has_function = true;
    self.on_data = OnData::Function(Box::new(func));
  }

  pub fn is_last(&self) -> bool {
    self.is_last
  }

  pub fn data(&mut self) -> &mut BytesMut {
    &mut self.data
  }

  pub(crate) fn init(&mut self, version: u8, method: String, uri: Uri, request_data: BytesMut) {
    self.data.clear();
    self.uri = uri;
    self.method = method;
    self.version = version;
    self.is_last = false;
    self.request_data = request_data;
  }

  pub(crate) fn add_header(&mut self, name: String, value: Vec<u8>) {
    // handle header addition
    self.headers.insert(name, value);
  }

  pub(crate) fn reset_headers(&mut self, len: usize) {
    self.headers.clear();
    if self.headers.capacity() < len {
      self.headers.reserve(len);
    }
  }
}
