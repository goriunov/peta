use net2::unix::UnixTcpBuilderExt;
use net2::TcpBuilder;

use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

pub mod reader;
pub mod request;
pub mod response;
pub mod status;
pub mod writer;

/// Reexports `tokio::prelude::*`
pub mod prelude {
  pub use tokio::prelude::*;
}

/// Reexports crate components
pub mod server {
  pub use crate::reader::Http;
  pub use crate::request::Request;
  pub use crate::response::Response;
  pub use crate::status::StatusMessage;
  pub use crate::Server;
}

/// Reexports `spawn` and `run` functions for `tokio::runtime::current_thread`
pub mod runtime {
  /// Spawn future on `tokio::runtime::current_thread`
  /// Slightly different from tokio's returns Result<(), ()>
  pub fn spawn<F>(future: F) -> Result<(), ()>
  where
    F: futures::Future<Item = (), Error = ()> + 'static,
  {
    tokio::runtime::current_thread::spawn(future);
    Ok(())
  }

  /// Run future on `tokio::runtime::current_thread`
  pub fn run<F>(future: F)
  where
    F: futures::Future<Item = (), Error = ()> + 'static,
  {
    let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
    runtime.spawn(future);
    runtime.run().expect("Could not run runtime");
  }
}

// need to write proper description for the server
pub struct Server {
  listener: TcpListener,
}

impl Server {
  /// Create new server which is running on `addr`
  pub fn new(addr: &'static str) -> Server {
    let listener = {
      let builder = TcpBuilder::new_v4().unwrap();
      // builder.reuse_address(true).unwrap();
      builder.reuse_port(true).unwrap();
      builder.bind(addr).unwrap();
      builder.listen(512).unwrap() // need to decide about backlog number
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
        // need to sort out error
        Err(_) => {}
      }
    }
  }
}
