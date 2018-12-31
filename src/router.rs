// implement router logic
// Now i can write nice router logic
use crate::request;
use crate::response;

use hashbrown;
use tokio::prelude::*;

use std::sync::Arc;

pub struct Node {
  path: String,
  children: Option<hashbrown::HashMap<String, Node>>,
}

type Func = Box<
  dyn Fn(request::Request) -> Box<dyn Future<Item = response::Response, Error = ()> + Send + Sync>
    + Send
    + Sync,
>;

pub struct Router {
  pub func: Option<Func>,
}

impl<'a> Router {
  pub fn new() -> Router {
    Router { func: None }
  }

  pub fn get<T>(&mut self, func: T)
  where
    T: Fn(request::Request) -> Box<Future<Item = response::Response, Error = ()> + Send + Sync>
      + Send
      + Sync
      + 'static,
  {
    self.func = Some(Box::new(func));
  }

  // pub fn prepare_run(
  //   &mut self,
  //   writer: tokio::io::WriteHalf<tokio::net::TcpStream>,
  //   item: reader::HttpReader<tokio::io::ReadHalf<tokio::net::TcpStream>>,
  // ) -> impl Future<Item = (), Error = ()> {
  //   // Box::new(
  //   let fut = item
  //     .map_err(|e| println!("Error is: {}", e))
  //     .fold(writer, |writer, b| {
  //       (self.func)(b).and_then(|res| res.write(writer).map_err(|e| println!("Error: {}", e)))
  //     })
  //     .map(|_| ());

  //   // tokio::spawn(fut);
  //   fut
  //   // )
  // }

  // pub fn call(
  //   &mut self,
  //   a: tokio::io::WriteHalf<tokio::net::TcpStream>,
  //   b: request::Request,
  // ) -> Box<Future<Item = tokio::io::WriteHalf<tokio::net::TcpStream>, Error = ()> + Send + 'static>
  // {
  //   (self.func)(a, b)
  // }
}

// type Func = Box<dyn FnMut(TcpStream) -> Box<Future<Item = TcpStream, Error = ()>>>;

// pub struct A {
//   func: Func,
// }

// impl A {
//   pub fn new(func: Func) -> A {
//     A { func }
//   }
// }

// pub struct Router <F, T> where{
//   fn: F
// }

// clean up useless impl
// pub struct Router<
//   T: Fn(
//     tokio::io::WriteHalf<tokio::net::TcpStream>,
//     request::Request,
//   ) -> Box<Future<Item = tokio::io::WriteHalf<tokio::net::TcpStream>, Error = ()>>,
// > {
//   cb: T,
// }

// pub fn my_fn<T, F>(f: F)
// where
//   F: FnMut(
//     T,
//     request::Request,
//   ) -> Box<Future<Item = tokio::io::WriteHalf<tokio::net::TcpStream>, Error = ()>>,
// {

// }

// impl<T> Router<T>
// where
//   T: Fn(
//     tokio::io::WriteHalf<tokio::net::TcpStream>,
//     request::Request,
//   ) -> Box<Future<Item = tokio::io::WriteHalf<tokio::net::TcpStream>, Error = ()>>,
// {
//   pub fn new(path: &str, cb: T) -> Router<T> {
//     Router { cb }
//   }

//   pub fn call(
//     self,
//     a: tokio::io::WriteHalf<tokio::net::TcpStream>,
//     b: request::Request,
//   ) -> Box<Future<Item = tokio::io::WriteHalf<tokio::net::TcpStream>, Error = ()>> {
//     self.call(a, b)
//   }

// pub fn add(&self, path: &str, cb: T)
// where
//   T: FnMut(
//     tokio::io::WriteHalf<tokio::net::TcpStream>,
//     request::Request,
//   ) -> Box<Future<Item = tokio::io::WriteHalf<tokio::net::TcpStream>, Error = ()>>,
// {

// }
// }
