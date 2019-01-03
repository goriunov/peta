use crate::method;
use crate::request;
use crate::response;

use hashbrown::HashMap;
use tokio::prelude::*;

use std::sync::Arc;

/// Abstraction of the return Boxed Future.
///
/// # Example
/// Each function which will be passed to `Router` must return
/// `ResponseFut`
/// ```
/// fn hello(req: Request) -> ResponseFut {
///   // rest of the code
/// }
///
/// router.get("/", hello)
/// ```
pub type ResponseFut = Box<dyn Future<Item = response::Response, Error = ()> + Send + Sync>;

/// Generates map between `path` and `method` which returns `ResponseFut`
/// supports `*` and `:` operators.
///
/// # Example
///
/// ```
/// let mut router = Router::new();
///
/// router.get("/", |req: Request| {
///   // do not forget to return ResponseFut
/// });
///
/// router.post("/home", |req: Request| {
///   // do not forget to return ResponseFut
/// });
///
/// // It is important to add default route
/// // can be simple 404 which will be called if nothing found
/// router.add_default(|req: Request| {});
///
/// let router = router.build();
///
/// ```
pub struct Router {
  get: Node,
  put: Node,
  post: Node,
  head: Node,
  patch: Node,
  delete: Node,
  options: Node,
  routes: Node,
  default: Option<StoreFunc>,
}

impl Router {
  /// Create instance of Router.
  ///
  /// ```
  /// let router = Router::new();
  /// ```
  pub fn new() -> Router {
    Router {
      default: None,
      // for now leave this one in
      routes: Node::default(),
      // create separate bucket for each method (not fun) :(
      get: Node::default(),
      put: Node::default(),
      post: Node::default(),
      head: Node::default(),
      patch: Node::default(),
      delete: Node::default(),
      options: Node::default(),
    }
  }

  /// Wrap router in `Arc` to be able to pass it across threads/components.
  ///
  /// ```
  /// let mut router = Router::new();
  /// // do some things with router variable
  ///
  /// let router = router.build();
  ///
  /// // now we can simply clone and call any functions from router instance
  /// let ref_router = router.clone();
  /// ```
  pub fn build(self) -> Arc<Self> {
    Arc::new(self)
  }

  /// Adds new `path -> method` map to the Router.
  ///
  /// ```
  /// router.add(method::GET, "/", |req: Request| {});
  /// router.add(method::POST, "/", |req: Request| {});
  /// // and so on
  /// ```
  pub fn add<F>(&mut self, method: &str, path: &'static str, func: F)
  where
    F: Fn(request::Request) -> ResponseFut + Send + Sync + 'static,
  {
    // use proper enum
    let mut node = match method {
      method::GET => &mut self.get,
      method::PUT => &mut self.put,
      method::POST => &mut self.post,
      method::HEAD => &mut self.head,
      method::PATCH => &mut self.patch,
      method::DELETE => &mut self.delete,
      method::OPTIONS => &mut self.options,
      _ => &mut self.routes,
    };

    match path {
      "/" => {
        // handle / case
        node.set_func(Box::new(func));
      }
      _ => {
        // handle rest of the cases
        for seg in path.split('/') {
          if !seg.is_empty() {
            let mut seg_arr = seg.chars();
            // check if path is param
            if seg_arr.next() == Some(':') {
              node = node.add_child(":", Some(seg_arr.as_str()));
              continue;
            }
            node = node.add_child(seg, None);
          }
        }

        node.set_func(Box::new(func));
      }
    }
  }

  /// Searches for appropriate `method` which is mapped to specific `path`
  ///
  /// ```
  /// let req: request::Request;
  /// // it will automatically extract path from `req`
  /// router.find(req)
  /// ```
  pub fn find(&self, mut req: request::Request) -> ResponseFut {
    let mut node = match req.method() {
      method::GET => &self.get,
      method::PUT => &self.put,
      method::POST => &self.post,
      method::HEAD => &self.head,
      method::PATCH => &self.patch,
      method::DELETE => &self.delete,
      method::OPTIONS => &self.options,
      _ => &self.routes,
    };

    // handle / route
    if req.uri().path() == "/" {
      return match node.method.as_ref() {
        Some(v) => (v)(req),
        None => (self.default.as_ref().unwrap())(req),
      };
    }

    let mut params: Vec<(&'static str, String)> = Vec::with_capacity(10);

    for seg in req.uri().path().split('/') {
      if !seg.is_empty() {
        if node.children.is_none() {
          // do default return
          return (self.default.as_ref().unwrap())(req);
        }

        let children = node.children.as_ref().unwrap();

        // find proper node with func
        let found_node = match children.get(seg) {
          Some(v) => v,
          None => {
            match children.get(":") {
              Some(v) => {
                params.push((v.param.unwrap(), seg.to_string()));
                v
              }
              // break from this function if we get in star
              None => match children.get("*") {
                Some(v) => {
                  // we need to attache params in here as we may and loop sooner
                  if !params.is_empty() {
                    req.set_params(Some(params));
                  }

                  return (v.method.as_ref().unwrap())(req);
                }
                None => {
                  // execute default if route not found at all
                  return (self.default.as_ref().unwrap())(req);
                }
              },
            }
          }
        };

        node = found_node;
      }
    }

    match node.method.as_ref() {
      Some(v) => {
        // set params only if it is not empty
        if !params.is_empty() {
          req.set_params(Some(params));
        }
        (v)(req)
      }
      None => (self.default.as_ref().unwrap())(req),
    }
  }

  /// Set default function for routes which were not mapped
  /// can be simple 404 response.
  ///
  /// ```
  /// router.add_default(|req: Request| {});
  /// ```
  pub fn add_default<F>(&mut self, func: F)
  where
    F: Fn(request::Request) -> ResponseFut + Send + Sync + 'static,
  {
    self.default = Some(Box::new(func));
  }
}

/// Abstracts `add` method by removing `method::*` param
impl Router {
  pub fn get<F>(&mut self, path: &'static str, func: F)
  where
    F: Fn(request::Request) -> ResponseFut + Send + Sync + 'static,
  {
    self.add(method::GET, path, func)
  }

  pub fn put<F>(&mut self, path: &'static str, func: F)
  where
    F: Fn(request::Request) -> ResponseFut + Send + Sync + 'static,
  {
    self.add(method::PUT, path, func)
  }

  pub fn post<F>(&mut self, path: &'static str, func: F)
  where
    F: Fn(request::Request) -> ResponseFut + Send + Sync + 'static,
  {
    self.add(method::POST, path, func)
  }

  pub fn head<F>(&mut self, path: &'static str, func: F)
  where
    F: Fn(request::Request) -> ResponseFut + Send + Sync + 'static,
  {
    self.add(method::HEAD, path, func)
  }

  pub fn patch<F>(&mut self, path: &'static str, func: F)
  where
    F: Fn(request::Request) -> ResponseFut + Send + Sync + 'static,
  {
    self.add(method::PATCH, path, func)
  }

  pub fn delete<F>(&mut self, path: &'static str, func: F)
  where
    F: Fn(request::Request) -> ResponseFut + Send + Sync + 'static,
  {
    self.add(method::DELETE, path, func)
  }

  pub fn options<F>(&mut self, path: &'static str, func: F)
  where
    F: Fn(request::Request) -> ResponseFut + Send + Sync + 'static,
  {
    self.add(method::OPTIONS, path, func)
  }
}

// probably will need to move it out of router component
type StoreFunc = Box<
  dyn Fn(request::Request) -> Box<dyn Future<Item = response::Response, Error = ()> + Send + Sync>
    + Send
    + Sync,
>;

struct Node {
  param: Option<&'static str>,
  method: Option<StoreFunc>,
  children: Option<HashMap<&'static str, Node>>,
}

impl Node {
  pub fn default() -> Node {
    Node {
      param: None,
      method: None,
      children: None,
    }
  }

  pub fn set_func(&mut self, func: StoreFunc) {
    self.method = Some(func);
  }

  pub fn add_child(&mut self, seg: &'static str, param: Option<&'static str>) -> &mut Node {
    if self.children.is_none() {
      self.children = Some(HashMap::new())
    }

    let node_map = self.children.as_mut().unwrap();

    // if key exist then return existing node ref
    if node_map.contains_key(seg) {
      return node_map.get_mut(seg).unwrap();
    }

    // create new if node
    node_map.insert(
      seg,
      Node {
        param,
        method: None,
        children: None,
      },
    );
    // this item is just added
    node_map.get_mut(seg).unwrap()
  }
}

// temp debug setter
impl std::fmt::Debug for Router {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "Router {{ \n   get: {:#?}, \npost: {:#?} \ndelete:{:#?}\nput:{:#?} \nroutes:{:#?} \n}}",
      self.get, self.post, self.delete, self.put, self.routes
    )
  }
}

impl std::fmt::Debug for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "Node {{ \n\tchildren: {:#?}, \n\tmethod: {:#?} \n\tparam:{:#?}\n}}",
      self.children,
      self.method.is_some(),
      self.param
    )
  }
}
