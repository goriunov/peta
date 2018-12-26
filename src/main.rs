use peta;

use tokio::net::TcpListener;
use tokio::prelude::*;

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

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(|sock| {
      let (read, write) = sock.split();

      let reader = peta::reader::HttpReader::new(read)
        .map_err(|e| println!("Error is: {}", e))
        .fold(write, |write, req| {
          let mut res = peta::response::Response::new();
          res.status("200 OK");
          // res.body_str("Hello world!");

          let user = Person {
            name: "Dmitrii".to_string(),
            last_name: "Hello".to_string(),
          };

          let json = serde_json::to_vec(&user).unwrap();
          res.body_vec(json);

          // println!("Is working");

          res.write(write).map_err(|e| println!("{}", e))
          // let status = "Hello world";

          // let body = req.body();
          // let version = req.version()
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
