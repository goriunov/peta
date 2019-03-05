use super::*;

pub struct Request {
  // internal use only
  pub(crate) on_data: OnData,
  pub(crate) has_data_function: bool,
  pub data: BytesMut,
  pub is_last: bool,
  headers: hashbrown::HashMap<String, Vec<u8>>,
  request_data: BytesMut,
  version: u8,
  method: Slice,
}

impl Request {
  pub fn new() -> Request {
    Request {
      has_data_function: false,
      is_last: false,
      method: (0, 0),
      version: 0,
      on_data: OnData::Empty,
      headers: hashbrown::HashMap::with_capacity(0),
      request_data: BytesMut::with_capacity(0),
      data: BytesMut::with_capacity(0),
    }
  }

  pub fn on_data<F>(&mut self, func: F)
  where
    F: Fn(ReqResTuple) -> ReturnFuture + Send + Sync + 'static,
  {
    self.has_data_function = true;
    self.on_data = OnData::Function(Box::new(func));
  }

  ///
  ///
  ///
  ///
  ///
  ///  Internal use function for request initialization
  pub(crate) fn init(&mut self, version: u8, method: Slice, request_data: BytesMut) {
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
