// implement router logic
use crate::request;
use crate::response;

use hashbrown;
use tokio::prelude::*;

use std::sync::Arc;

pub type ReturnFuture = Box<dyn Future<Item = response::Response, Error = ()> + Send + Sync>;

type StoreFunc = Box<
  dyn Fn(request::Request) -> Box<dyn Future<Item = response::Response, Error = ()> + Send + Sync>
    + Send
    + Sync,
>;

pub struct Node {
  param: Option<&'static str>,
  method: Option<StoreFunc>,
  children: Option<hashbrown::HashMap<&'static str, Node>>,
}

impl std::fmt::Debug for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "Node {{ \nchildren: {:#?}, \nmethod: {:#?} \nparam:{:#?}\n}}",
      self.children,
      self.method.is_some(),
      self.param
    )
  }
}

#[derive(Debug)]
pub struct Router {
  routes: Node,
}

impl Router {
  pub fn new() -> Router {
    Router {
      routes: Node {
        param: None,
        method: None,
        children: None,
      },
    }
  }

  pub fn build(self) -> Arc<Self> {
    Arc::new(self)
  }

  pub fn find(&self, req: request::Request) -> ReturnFuture {
    // !! we need to do a lot of optimization for search
    // and add additional router parsing things
    let mut node = &self.routes;

    for seg in req.uri().path().split('/') {
      if seg.len() > 0 {
        if node.children.is_none() {
          break;
        }

        let children = node.children.as_ref().unwrap();

        let mut found_node = children.get(seg);

        if found_node.is_none() {
          // if we found at least star then load star route
          found_node = children.get("*");
          if found_node.is_some() {
            node = found_node.unwrap();
          }

          break;
        }

        node = found_node.unwrap();
      }
    }

    match node.method.as_ref() {
      Some(func) => (func)(req),
      None => {
        // if none then load 404 route
        (self.routes.method.as_ref().unwrap())(req)
      }
    }
  }

  pub fn add<F>(&mut self, path: &'static str, func: F)
  where
    F: Fn(request::Request) -> Box<Future<Item = response::Response, Error = ()> + Send + Sync>
      + Send
      + Sync
      + 'static,
  {
    let mut node = &mut self.routes;

    if path == "*" {
      set_func(node, Box::new(func));
      // if it is stark then set default
      return;
    }

    for item in path.split('/') {
      if item.len() > 0 {
        // check if route is param route
        let mut path_chars = item.chars();

        if path_chars.next() == Some(':') {
          add_node(node, ":", Some(path_chars.as_str()));
          node = node.children.as_mut().unwrap().get_mut(":").unwrap();
        } else {
          add_node(node, item, None);
          node = node.children.as_mut().unwrap().get_mut(item).unwrap();
        }
      }
    }

    set_func(node, Box::new(func));
  }
}

fn set_func(node: &mut Node, func: StoreFunc) {
  node.method = Some(func);
}

fn add_node(node: &mut Node, path: &'static str, param: Option<&'static str>) {
  // if no children exist then create new one
  if node.children.is_none() {
    let mut hash = hashbrown::HashMap::new();
    hash.insert(
      path,
      Node {
        param,
        method: None,
        children: None,
      },
    );
    node.children = Some(hash);
  } else {
    // if we have children then reuse existing or add new
    let children = node.children.as_mut().unwrap();
    if children.get(path).is_none() {
      children.insert(
        path,
        Node {
          param,
          method: None,
          children: None,
        },
      );
    }
  }
}
