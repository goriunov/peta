use tokio::net::TcpListener;
use tokio::prelude::*;

use peta::server::{method, status, HttpReader, Request, Response, ReturnFuture, Router};

use std::time::{Duration, Instant};
use tokio::timer::Delay;

// use http::Uri;

// Test json response
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Person {
  name: String,
  last_name: String,
}

fn home(req: Request) -> ReturnFuture {
  let mut res = Response::new();
  res.status(status::OK);
  res.body_str(req.uri().path());

  // need to abstract response way
  Box::new(futures::future::ok(res))
}

fn delay(req: Request) -> ReturnFuture {
  let when = Instant::now() + Duration::from_millis(2000);

  let delay = Delay::new(when)
    .map_err(|e| panic!("Delay errored; err={:?}", e))
    .and_then(move |_| {
      let mut res = Response::new();
      res.status(status::OK);
      res.body_str("Hello world!");
      Ok(res)
    });

  Box::new(delay)
}

fn hello_world(req: Request) -> ReturnFuture {
  // println!("{:?}", req.params());
  let mut res = Response::new();
  res.status(status::OK);
  res.body_str("Hello world");

  Box::new(futures::future::ok(res))
}

fn not_found(req: Request) -> ReturnFuture {
  let mut res = Response::new();
  res.status(status::OK);
  res.body_str("Did not found page");

  Box::new(futures::future::ok(res))
}

fn main() {
  // find a wait to pass state !!
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

  // build router
  let mut router = Router::new();
  // does not take routes order in account yet
  router.add(method::GET, "/hello", hello_world);
  router.add(method::GET, "/home", home);
  router.add(method::GET, "/delay", delay);
  router.add(method::GET, "/delay/*", home);
  // we must provide "*" route // as a default response
  router.add(method::GET, "*", not_found);
  router.add(method::GET, "/hello/:world", hello_world);

  println!("{:#?}", router);
  // will need to thing what is better
  let router = router.build();

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(move |sock| {
      let (read, write) = sock.split();

      // get arc pointer
      let router = router.clone();
      let reader = HttpReader::new(read)
        .map_err(|e| println!("Error is: {}", e))
        .fold(write, move |writer, req| {
          router
            .find(req)
            .and_then(|rsp| rsp.write(writer).map_err(|e| println!("Error: {}", e)))
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
