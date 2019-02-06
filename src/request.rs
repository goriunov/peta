use bytes::{BufMut, BytesMut};
use tokio::prelude::*;

use super::response;

type ReturnFuture =
  Box<dyn Future<Item = ((Request, response::Response)), Error = std::io::Error> + Send + Sync>;
pub(crate) type Slice = (usize, usize);

pub(crate) enum OnData {
  Empty,
  Function(Box<Fn((Request, response::Response)) -> ReturnFuture + Send + Sync>),
}

pub struct Request {
  pub(crate) on_data: OnData,
  pub(crate) request_raw: BytesMut,
  pub(crate) headers: Vec<(String, Slice)>,
  pub(crate) version: u8,
  pub(crate) method: Slice,
  pub(crate) data: BytesMut,
  pub(crate) is_last: bool,
}

impl Request {
  pub fn new() -> Request {
    Request {
      is_last: false,
      data: BytesMut::with_capacity(0),
      on_data: OnData::Empty,
      headers: Vec::with_capacity(0),
      request_raw: BytesMut::with_capacity(0),
      version: 0,
      method: (0, 0),
    }
  }

  pub fn data(&self) -> &BytesMut {
    &self.data
  }

  pub fn data_take(&mut self) -> BytesMut {
    self.data.take()
  }

  pub fn is_last(&self) -> bool {
    self.is_last
  }

  pub fn on_data<F>(&mut self, func: F)
  where
    F: Fn((Request, response::Response)) -> ReturnFuture + Send + Sync + 'static,
  {
    self.on_data = OnData::Function(Box::new(func))
  }
}
