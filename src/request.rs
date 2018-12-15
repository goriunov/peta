pub struct Request {
  // body: String,
  method: String,
  path: String,
  // header: Vec<u8>,
  // version: String,
}

impl Request {
  pub fn new(path: String, method: String) -> Request {
    Request { path, method }
  }

  pub fn method(&self) -> &str {
    self.method.as_str()
  }

  pub fn path(&self) -> &str {
    self.path.as_str()
  }
}
