use tokio::prelude::*;

use super::request;
use super::response;

type ReturnFuture = Box<
  dyn Future<Item = ((request::Request, response::Response)), Error = std::io::Error> + Send + Sync,
>;

pub trait FunctionCall {
  fn find(&self, data: (request::Request, response::Response)) -> ReturnFuture;
}

pub struct Router<F> {
  default: F,
}

impl<F> Router<F> {
  pub fn new(default: F) -> Router<F>
  where
    F: Fn((request::Request, response::Response)) -> ReturnFuture + Send + Sync,
  {
    Router { default }
  }
}

impl<F> FunctionCall for Router<F>
where
  F: Fn((request::Request, response::Response)) -> ReturnFuture + Send + Sync,
{
  fn find(&self, (req, res): (request::Request, response::Response)) -> ReturnFuture {
    (self.default)((req, res))
  }
}
