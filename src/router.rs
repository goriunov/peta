// implement router logic
// Now i can write nice router logic
use crate::request;
use crate::response;

use hashbrown;
use tokio::prelude::*;

use std::sync::Arc;

type StoreFunc = Box<
  dyn Fn(request::Request) -> Box<dyn Future<Item = response::Response, Error = ()> + Send + Sync>
    + Send
    + Sync,
>;

pub struct Node {
  path: String,
  children: Option<hashbrown::HashMap<String, Node>>,
}

pub struct Router {
  pub maps: Node,
  pub func: Option<StoreFunc>,
}

impl<'a> Router {
  pub fn new() -> Router {
    Router {
      maps: Node {
        path: String::from("MYPATH"),
        children: None,
      },
      func: None,
    }
  }

  pub fn get<F>(mut self, func: F) -> Router
  where
    F: Fn(request::Request) -> Box<Future<Item = response::Response, Error = ()> + Send + Sync>
      + Send
      + Sync
      + 'static,
  {
    self.func = Some(Box::new(func));
    self
  }

  pub fn build(self) -> Arc<Self> {
    Arc::new(self)
  }
}
