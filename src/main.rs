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

fn main() {
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

  let mut router = peta::router::Router::new();

  router.add(String::from("/home"), move |req| {
    // println!("Query {}", req.uri().query().unwrap_or("None"));
    // println!("Path {}", req.method());
    let mut res = peta::response::Response::new();
    res.status(peta::status::OK);
    res.body_str(req.uri().path());

    Box::new(futures::future::ok(res))
  });

  router.add(String::from("/delay"), move |req| {
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
  });
  // build router
  let router = router.build();

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(move |sock| {
      let (read, write) = sock.split();

      // get arc pointer
      let router = router.clone();
      // need to write boxed future
      let reader = peta::reader::HttpReader::new(read)
        .map_err(|e| println!("Error is: {}", e))
        .fold(write, move |writer, req| {
          // let mut res = peta::response::Response::new();
          // res.status(peta::status::OK);
          // res.body_str(req.uri().path());

          match router.find(req.uri().path()) {
            Some(func) => (func)(req),
            None => {
              (router.find("/home").unwrap())(req)

              // let mut res = peta::response::Response::new();
              // res.status(peta::status::OK);
              // res.body_str(req.uri().path());
              // Box::new(futures::future::ok(res))
              //   .and_then(|res| res.write(writer).map_err(|e| println!("Error: {}", e)))
            }
          }
          .and_then(|rsp| rsp.write(writer).map_err(|e| println!("Error: {}", e)))

          // res.write(writer).map_err(|e| println!("Error: {}", e))
          // (router.func.as_ref().unwrap())(req)
          //   .and_then(|rsp| rsp.write(writer).map_err(|e| println!("Error: {}", e)))
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

// fn on_delay(
//   res: peta::request::Request,
// ) -> Box<Future<Item = peta::response::Response, Error = ()> + Send + Sync> {
//   let when = Instant::now() + Duration::from_millis(2000);

//   let delay = Delay::new(when)
//     .map_err(|e| panic!("Delay errored; err={:?}", e))
//     .and_then(move |_| {
//       let mut res = peta::response::Response::new();
//       res.status(peta::status::OK);
//       res.body_str("Hello world!");
//       Ok(res)
//     });

//   Box::new(delay)
// }

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
