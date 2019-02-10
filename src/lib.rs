// common imports
pub use bytes::{BufMut, BytesMut};
pub use tokio::prelude::*;

// modules (for now public)
pub mod reader;
pub mod request;
pub mod response;

// common types
pub(crate) type Slice = (usize, usize);
pub(crate) type ReadHalf = tokio::io::ReadHalf<tokio::net::TcpStream>;
pub(crate) type WriteHalf = tokio::io::WriteHalf<tokio::net::TcpStream>;
pub(crate) type ReturnFuture = Box<
  dyn Future<Item = ((request::Request, response::Response)), Error = std::io::Error> + Send + Sync,
>;
