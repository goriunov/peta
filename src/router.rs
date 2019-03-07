use super::*;

pub struct Router<F> {
  default: Option<F>,
}

impl<F> Router<F> {
  pub fn new() -> Router<F> {
    Router { default: None }
  }

  pub fn get(&mut self, _string: &str, default: F)
  where
    F: Fn(ReqResTuple) -> ReturnFuture + Send + Sync,
  {
    self.default = Some(default);
  }
}

impl<F> RouterSearch for Router<F>
where
  F: Fn(ReqResTuple) -> ReturnFuture + Send + Sync,
{
  fn find(&self, data: ReqResTuple) -> ReturnFuture {
    (self.default.as_ref().unwrap())(data)
  }
}
