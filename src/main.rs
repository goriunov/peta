use peta;

use tokio::net::TcpListener;
use tokio::prelude::*;

use std::time::{Duration, Instant};
use tokio::timer::Delay;

use http::Uri;

// Test json response
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Person {
  name: String,
  last_name: String,
}

fn home(req: peta::request::Request) -> peta::router::ReturnFuture {
  let mut res = peta::response::Response::new();
  res.status(peta::status::OK);
  res.body_str(req.uri().path());

  Box::new(futures::future::ok(res))
}

fn delay(req: peta::request::Request) -> peta::router::ReturnFuture {
  let when = Instant::now() + Duration::from_millis(2000);

  let delay = Delay::new(when)
    .map_err(|e| panic!("Delay errored; err={:?}", e))
    .and_then(move |_| {
      let mut res = peta::response::Response::new();
      res.status(peta::status::OK);
      res.body_str("Hello world!");
      Ok(res)
    });

  Box::new(delay)
}

fn not_found(req: peta::request::Request) -> peta::router::ReturnFuture {
  let mut res = peta::response::Response::new();
  res.status(peta::status::OK);
  res.body_str("Did not found page");

  Box::new(futures::future::ok(res))
}

fn main() {
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

  // build router
  let mut router = peta::router::Router::new();
  // does not take routes order in account yet
  router.add("/home", home);
  router.add("/delay", delay);
  router.add("/delay/*", home);
  // we must provide "*" route // as a default response
  router.add("*", not_found);
  let router = router.build();

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(move |sock| {
      let (read, write) = sock.split();

      // get arc pointer
      let router = router.clone();
      let reader = peta::reader::HttpReader::new(read)
        .map_err(|e| println!("Error is: {}", e))
        .fold(write, move |writer, req| {
          router.find(req.uri().path())(req)
            .and_then(|resp| resp.write(writer).map_err(|e| println!("Error: {}", e)))
        })
        .map(|_| ());

      tokio::runtime::current_thread::spawn(reader);
      // tokio::spawn(reader);
      Ok(())
    });

  runtime.spawn(server);
  runtime.run().unwrap();

  // tokio::run(server);
}
