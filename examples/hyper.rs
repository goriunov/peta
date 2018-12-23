#![deny(warnings)]
extern crate hyper;

use hyper::rt::{self, Future};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};

// use hyper::header::HeaderValue;

fn main() {
  // pretty_env_logger::init();
  let addr = ([127, 0, 0, 1], 3001).into();

  let server = Server::bind(&addr)
    .serve(|| {
      // This is the `Service` that will handle the connection.
      // `service_fn_ok` is a helper to convert a function that
      // returns a Response into a `Service`.
      service_fn_ok(move |_: Request<Body>| {
        let res = Response::new(Body::from("Hello World!"));
        // res
        //   .headers_mut()
        // .insert("Connection", HeaderValue::from_static("close"));

        res
      })
    })
    .map_err(|e| eprintln!("server error: {}", e));

  println!("Listening on http://{}", addr);

  rt::run(server);
}
