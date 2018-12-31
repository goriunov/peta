use peta;

use tokio::net::TcpListener;
use tokio::prelude::*;

use std::sync::Arc;

use std::time::{Duration, Instant};
use tokio::timer::Delay;

// Test json response
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Person {
  name: String,
  last_name: String,
}

fn main() {
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

  let mut router = peta::router::Router::new();

  router.get(move |req| {
    // example with using delay future
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
    // let mut res = peta::response::Response::new();
    // res.status(peta::status::OK);
    // res.body_str("Hello world!");

    // Box::new(futures::future::ok(res))
  });

  // router.get()
  // hide ark somewhere to make it nicer
  let router = Arc::new(router);

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(move |sock| {
      let (read, write) = sock.split();

      let router = Arc::clone(&router);
      // need to write boxed future
      let reader = peta::reader::HttpReader::new(read)
        .map_err(|e| println!("Error is: {}", e))
        .fold(write, move |writer, req| {
          // make this desing nicer
          (router.func.as_ref().unwrap())(req)
            .and_then(|rsp| rsp.write(writer).map_err(|e| println!("Error: {}", e)))
        })
        .map(|_| ());

      tokio::runtime::current_thread::spawn(reader);
      // tokio::spawn(reader);
      Ok(())
    });

  runtime.spawn(server);
  runtime.run();

  // tokio::run(server);
}

// fn process<'a, S: AsyncWrite + 'a>(
//   write: S,
//   req: peta::request::Request,
// ) -> Box<Future<Item = S, Error = ()> + 'a> {
//   let mut res = peta::response::Response::new();
//   res.status(peta::status::OK);
//   res.body_str("Hello world!");

//   // let user = Person {
//   //   name: "Dmitrii".to_string(),
//   //   last_name: "Hello".to_string(),
//   // };

//   // let json = serde_json::to_vec(&user).unwrap();
//   // res.body_vec(json);

//   // println!("Is working");

//   Box::new(res.write(write).map_err(|e| println!("Error: {}", e)))
// }
