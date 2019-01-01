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
  method: Option<StoreFunc>,
  children: Option<hashbrown::HashMap<String, Node>>,
}

impl std::fmt::Debug for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "Node {{ \n children: {:#?} \n}}", self.children)
  }
}

#[derive(Debug)]
pub struct Router {
  routes: Option<Node>,
}

impl Router {
  pub fn new() -> Router {
    Router { routes: None }
  }

  pub fn build(self) -> Arc<Self> {
    Arc::new(self)
  }

  pub fn find(&self, path: &str) -> Option<&StoreFunc> {
    let mut node = self.routes.as_ref().unwrap();
    for item in path.split('/') {
      match &node.children {
        Some(found) => match found.get(&String::from(item)) {
          Some(exist) => {
            node = exist;
          }
          None => {}
        },
        None => {}
      }
    }

    // match node.method {
    //   Some(f) => f.as_ref(),
    //   None
    // }

    node.method.as_ref()
  }

  pub fn add<F>(&mut self, path: String, func: F)
  where
    F: Fn(request::Request) -> Box<Future<Item = response::Response, Error = ()> + Send + Sync>
      + Send
      + Sync
      + 'static,
  {
    match &self.routes {
      Some(node) => {}
      None => {
        self.routes = Some(Node {
          method: None,
          children: None,
        });
      }
    }

    let mut node = self.routes.as_mut().unwrap();
    // let mut node: Option<Node> = None;

    for item in path.split('/') {
      if item.len() > 0 {
        Router::loop_add(node, String::from(item));
        node = node.children.as_mut().unwrap().get_mut(item).unwrap();
      }
    }

    Router::set_fn(node, func);

    println!("{:#?}", self);
  }

  fn set_fn<F>(node: &mut Node, func: F)
  where
    F: Fn(request::Request) -> Box<Future<Item = response::Response, Error = ()> + Send + Sync>
      + Send
      + Sync
      + 'static,
  {
    node.method = Some(Box::new(func));
  }

  // can be optimized
  fn loop_add(node: &mut Node, path: String) {
    match &mut node.children {
      Some(children) => match children.get(path.as_str()) {
        Some(_) => {}
        None => {
          children.insert(
            path,
            Node {
              method: None,
              children: None,
            },
          );
        }
      },
      None => {
        let mut hash = hashbrown::HashMap::new();
        hash.insert(
          path,
          Node {
            method: None,
            children: None,
          },
        );

        node.children = Some(hash);
      }
    }
  }
}

///
///
///
///
///
///
/// IGNORE OLD IMPLEMNTATION
///
///
///
///
///
///
// Old router implementation
pub struct Router2 {
  // pub maps: Node,
  pub func: Option<StoreFunc>,
}

impl<'a> Router2 {
  pub fn new() -> Router2 {
    Router2 {
      // maps: Node {
      //   path: String::from("MYPATH"),
      //   children: None,
      // },
      func: None,
    }
  }

  pub fn get<F>(mut self, func: F) -> Router2
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
