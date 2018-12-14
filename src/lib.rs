use net2::unix::UnixTcpBuilderExt;
use net2::TcpBuilder;

use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

pub mod reader;
pub mod response;
pub mod writer;

// some prelude to use futures stream
pub mod prelude {
  // pub use http::*;
  pub use tokio::prelude::*;

}

// simple current runtime
pub mod runtime {
  pub use tokio::runtime::current_thread::spawn;

  pub fn run<F>(future: F)
  where
    F: futures::Future<Item = (), Error = ()> + 'static,
  {
    let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
    runtime.spawn(future);
    runtime.run().expect("Could not run runtime");
  }
}

///
///
///
///
///
///
///
///
///
/// below things is not important part

pub struct Server {
  listener: TcpListener,
}

impl Server {
  pub fn new(addr: &'static str) -> Server {
    let listener = {
      let builder = TcpBuilder::new_v4().unwrap();
      // builder.reuse_address(true).unwrap();
      builder.reuse_port(true).unwrap();
      builder.bind(addr).unwrap();
      builder.listen(128).unwrap() // need to decide about backlog number
    };
    let listener = TcpListener::from_std(listener, &tokio::reactor::Handle::current()).unwrap();
    Server { listener }
  }
}

impl Stream for Server {
  type Item = TcpStream;
  type Error = ();

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    loop {
      match self.listener.poll_accept() {
        Ok(Async::Ready((socket, _))) => {
          return Ok(Async::Ready(Some(socket)));
        }
        Ok(Async::NotReady) => {
          return Ok(Async::NotReady);
        }
        Err(e) => {
          println!("Got error {}", e);
        }
      }
    }
  }
}
