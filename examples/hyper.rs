#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate tokio;

use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};
use tokio::runtime::current_thread;

fn main() {
  let addr = ([127, 0, 0, 1], 3001).into();

  // Using a !Send request counter is fine on 1 thread...

  let new_service = move || {
    // For each connection, clone the counter to use in our service...
    service_fn_ok(move |_| Response::new(Body::from("Hello world")))
  };

  // Since the Server needs to spawn some background tasks, we needed
  // to configure an Executor that can spawn !Send futures...
  let exec = current_thread::TaskExecutor::current();

  let server = Server::bind(&addr)
    .executor(exec)
    .serve(new_service)
    .map_err(|e| eprintln!("server error: {}", e));

  println!("Listening on http://{}", addr);

  current_thread::Runtime::new()
    .expect("rt new")
    .spawn(server)
    .run()
    .expect("rt run");
}
