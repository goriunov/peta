use bytes::BytesMut;
use tokio::prelude::*;

pub type Slice = (usize, usize);

pub struct Request {
  pub data: BytesMut,
  pub headers: Vec<(String, Slice)>,
  pub method: Slice,
}

impl Request {}
// pub fn new() -> Request {
//   Request {
//     data: None,
//     headers: Vec::new(),
//     body: (0, 0),
//     method: (0, 0),
//   }
// }
// still need to add it
// }

// pub struct DataStream {
//   is_notified: bool,
//   task: futures::task::Task,
// }
// impl DataStream {
//   pub fn new() -> DataStream {
//     DataStream {
//       task: futures::task::current(),
//       is_notified: false,
//     }
//   }

//   pub fn get_task(&self) -> futures::task::Task {
//     futures::task::current()
//   }

//   // pub fn notify(&mut self) {
//   //   self.task.notify();
//   //   self.is_notified = true;
//   // }
// }
// impl Stream for DataStream {
//   type Item = ();
//   type Error = std::io::Error;

//   fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
//     // self.ta
//     // if self.is_notified {
//     // println!("Has been notified");
//     // }
//     // self.is_notified = true;

//     // loop {
//     return Ok(Async::NotReady);
//     // println!("Has been notified");
//     // }
//   }
// }
