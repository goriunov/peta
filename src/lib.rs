// common imports
pub use bytes::{BufMut, BytesMut};
pub use hashbrown;
pub use tokio::prelude::*;

// modules (for now public)
pub mod chunk;
pub mod date;
pub mod reader;
pub mod request;
pub mod response;
pub mod router;
pub mod writer;

// common types
pub(crate) type Slice = (usize, usize);
pub(crate) type ReadHalf = tokio::io::ReadHalf<tokio::net::TcpStream>;
pub(crate) type WriteHalf = tokio::io::WriteHalf<tokio::net::TcpStream>;
pub(crate) type ReqResTuple = (request::Request, response::Response);
pub(crate) type ReturnFuture =
  Box<dyn Future<Item = ReqResTuple, Error = std::io::Error> + Send + Sync>;

pub enum OnData {
  Empty,
  Function(Box<Fn(ReqResTuple) -> ReturnFuture + Send + Sync>),
}

pub trait RouterSearch {
  fn find(&self, data: ReqResTuple) -> ReturnFuture;
}
