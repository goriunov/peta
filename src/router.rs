use super::*;

type StoreFunc = Box<dyn Fn(ReqResTuple) -> ReturnFuture + Send + Sync>;

pub struct Router {
  routes: Node,
}

impl Router {
  pub fn new() -> Router {
    Router {
      routes: Node::new(),
    }
  }

  pub fn add<F>(&mut self, method: &str, path: &'static str, func: F)
  where
    F: Fn(ReqResTuple) -> ReturnFuture + Send + Sync + 'static,
  {
    let mut node = &mut self.routes;
    match path {
      "/" => node.set_method(Box::new(func)),
      _ => {
        // implement all the rest
        for seg in path.split('/') {
          if !seg.is_empty() {
            node = node.add_child(seg);
          }
        }

        node.set_method(Box::new(func));
      }
    };
  }
}

impl RouterSearch for Router {
  fn find(&self, (req, res): ReqResTuple) -> ReturnFuture {
    // TODO: Clean up this function from trash
    // search for correct Node
    let mut node = &self.routes;

    if req.uri.path() == "/" {
      // handle uri
      return match node.method.as_ref() {
        Some(v) => (v)((req, res)),
        None => Box::new(res.write("".as_bytes()).map(|res| ((req, res)))),
      };
    }

    // handle all the stuff
    for seg in req.uri.path().split('/') {
      if !seg.is_empty() {
        if node.children.is_empty() {
          return Box::new(res.write("".as_bytes()).map(|res| ((req, res))));
        }

        node = match node.children.get(seg) {
          Some(v) => v,
          None => return Box::new(res.write("".as_bytes()).map(|res| ((req, res)))),
        }
      }
    }

    return match node.method.as_ref() {
      Some(v) => (v)((req, res)),
      None => Box::new(res.write("".as_bytes()).map(|res| ((req, res)))),
    };
  }
}

// Implement single Node
struct Node {
  pub(crate) method: Option<StoreFunc>,
  pub(crate) children: hashbrown::HashMap<&'static str, Node>,
}

impl Node {
  pub fn new() -> Node {
    Node {
      method: None,
      children: hashbrown::HashMap::new(),
    }
  }

  pub fn set_method(&mut self, func: StoreFunc) {
    self.method = Some(func);
  }

  pub fn add_child(&mut self, seg: &'static str) -> &mut Node {
    if !self.children.contains_key(seg) {
      self.children.insert(
        seg,
        Node {
          method: None,
          children: hashbrown::HashMap::new(),
        },
      );
    }

    return self.children.get_mut(seg).unwrap();
  }
}
