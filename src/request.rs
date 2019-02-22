use super::*;

pub struct Request {
  pub(crate) has_on_data: bool,
  pub(crate) on_data: OnData,
  pub data: BytesMut,
}

impl Request {
  pub fn new() -> Request {
    Request {
      has_on_data: false,
      on_data: OnData::Empty,
      data: BytesMut::with_capacity(0),
    }
  }

  pub fn on_data<F>(&mut self, func: F)
  where
    F: Fn(ReqResTuple) -> ReturnFuture + Send + Sync + 'static,
  {
    self.has_on_data = true;
    self.on_data = OnData::Function(Box::new(func));
  }
}
