use super::*;

pub struct Router<F> {
  default: F,
}

impl<F> Router<F> {
  pub fn new(default: F) -> Router<F>
  where
    F: Fn(ReqResTuple) -> ReturnFuture + Send + Sync,
  {
    Router { default }
  }
}

impl<F> RouterSearch for Router<F>
where
  F: Fn(ReqResTuple) -> ReturnFuture + Send + Sync,
{
  fn find(&self, data: ReqResTuple) -> ReturnFuture {
    (self.default)(data)
  }
}
