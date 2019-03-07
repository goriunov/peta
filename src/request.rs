use super::*;

pub struct Request {
  // internal use only
  pub(crate) on_data: OnData,
  pub(crate) has_data_function: bool,
  pub(crate) data: BytesMut,
  pub(crate) is_last: bool,
  method: String,
  version: u8,
  request_data: BytesMut,
  headers: hashbrown::HashMap<String, Vec<u8>>,
}

impl Request {
  pub fn new() -> Request {
    Request {
      has_data_function: false,
      is_last: false,
      version: 0,
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
    self.has_data_function = true;
    self.on_data = OnData::Function(Box::new(func));
  }

  pub fn is_last(&self) -> bool {
    self.is_last
  }

  pub(crate) fn init(&mut self, version: u8, method: String, request_data: BytesMut) {
    self.data.clear();
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
