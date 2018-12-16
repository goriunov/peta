pub struct Request {
  // body: String,
  path: String,
  method: String,
  version: String,
  // header: Vec<u8>,
}

impl Request {
  pub fn new(path: String, method: String, version: String) -> Request {
    Request {
      path,
      method,
      version,
    }
  }

  pub fn method(&self) -> &str {
    self.method.as_str()
  }

  pub fn path(&self) -> &str {
    self.path.as_str()
  }

  pub fn version(&self) -> &str {
    self.path.as_str()
  }
}
