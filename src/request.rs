use bytes::BytesMut;
use tokio::prelude::*;

pub type Slice = (usize, usize);

pub struct Content {
  pub data: Option<BytesMut>,
  pub headers: Vec<(String, Slice)>,
  pub body: Slice,
  pub method: Slice,
}

pub struct Request {
  // operation properties
  pub(crate) amt: usize,
  pub(crate) body_size: usize,
  pub(crate) is_chunked: bool,
  pub(crate) is_waiting: bool,
  pub on_data: DataStream,
  // user accessible properties
  pub content: Option<Content>,
}

impl Request {
  pub fn new() -> Request {
    Request {
      amt: 0,
      body_size: 0,
      is_chunked: false,
      is_waiting: false,
      on_data: DataStream::new(),
      content: None,
    }
  }
  // still need to add it
}

pub struct DataStream {
  is_notified: bool,
  task: futures::task::Task,
}
impl DataStream {
  pub fn new() -> DataStream {
    DataStream {
      task: futures::task::current(),
      is_notified: false,
    }
  }

  pub fn get_task(&self) -> futures::task::Task {
    futures::task::current()
  }

  // pub fn notify(&mut self) {
  //   self.task.notify();
  //   self.is_notified = true;
  // }
}
impl Stream for DataStream {
  type Item = ();
  type Error = std::io::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    // self.ta
    // if self.is_notified {
    // println!("Has been notified");
    // }
    // self.is_notified = true;

    // loop {
    return Ok(Async::NotReady);
    // println!("Has been notified");
    // }
  }
}
