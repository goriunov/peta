use bytes::BytesMut;

pub type Slice = (usize, usize);

pub struct Content {
  pub data: Option<BytesMut>,
  pub headers: Vec<(String, Slice)>,
  pub body: Slice,
  pub method: Slice,
}

pub struct Request {
  // operation properties
  pub(crate) amt: usize,
  pub(crate) body_size: usize,
  pub(crate) is_chunked: bool,
  pub(crate) is_waiting: bool,
  // user accessible properties
  pub content: Option<Content>,
}

impl Request {
  pub fn new() -> Request {
    Request {
      amt: 0,
      body_size: 0,
      is_chunked: false,
      is_waiting: false,
      content: None,
    }
  }
  // still need to add it
}
