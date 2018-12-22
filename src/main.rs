use peta::server::{Http, Response, Server, StatusMessage};
use peta::{prelude::*, runtime};

// for timer example
use std::time::{Duration, Instant};
use tokio::timer::Delay;

fn main() {
  let addr = "0.0.0.0:3000";

  let server = Server::new(&addr)
    .map_err(|e| println!("failed to accept socket; error = {:?}", e))
    .for_each(|socket| {
      let (read, write) = socket.split();

      let conn = Http::new(read)
        .fold(write, |write, req| {
          let path = req.path();

          // Read headers
          // for header in req.headers() {
          //   println!(
          //     "Header: {:?}: {:?}",
          //     header.0,
          //     std::str::from_utf8(header.1).unwrap()
          //   );
          // }

          let rsp = Response::new().header("Content-Type", "text/plain");

          match path {
            "/" => hello_world(rsp),
            "/delay" => delay(rsp),
            _ => not_found(rsp),
          }
          .and_then(move |rsp| rsp.write(write))
        })
        .map_err(|e| println!("Error in http parsing; err={:?}", e))
        .map(|_| ());

      // spawn each connection
      runtime::spawn(conn)
    });

  println!("Server is listening on {}", addr);
  runtime::run(server);
}

// hello world example
pub fn hello_world(rsp: Response) -> Box<Future<Item = Response, Error = std::io::Error>> {
  let hello = futures::future::ok(rsp.status(StatusMessage::OK).body("Hello world"));
  Box::new(hello)
}

// delay example
pub fn delay(rsp: Response) -> Box<Future<Item = Response, Error = std::io::Error>> {
  let when = Instant::now() + Duration::from_millis(2000);

  let delay = Delay::new(when)
    .map_err(|e| panic!("Delay errored; err={:?}", e))
    .and_then(move |_| {
      Ok(
        rsp
          .status(StatusMessage::OK)
          .body("Got in delay page 2000ms"),
      )
    });
  Box::new(delay)
}

// default not found response
pub fn not_found(rsp: Response) -> Box<Future<Item = Response, Error = std::io::Error>> {
  let at404 = futures::future::ok(rsp.status(StatusMessage::NOT_FOUND).body("Could not find"));
  Box::new(at404)
}
