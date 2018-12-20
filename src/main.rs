use simple_http::prelude::*;
use simple_http::{
  reader::HttpReader, response::Response, response::StatusMessage, runtime, Server,
};

// for timer example
use std::time::{Duration, Instant};
use tokio::timer::Delay;

fn main() {
  let addr = "0.0.0.0:3000";

  let server = Server::new(&addr)
    .map_err(|e| println!("failed to accept socket; error = {:?}", e))
    .for_each(|socket| {
      let (reader, writer) = socket.split();

      let conn = HttpReader::new(reader)
        .fold(writer, |writer, req| {
          let path = req.path();

          // for header in req.headers() {
          //   println!(
          //     "Header: {:?}: {:?}",
          //     header.0,
          //     std::str::from_utf8(header.1).unwrap()
          //   );
          // }

          let rsp = Response::new()
            .status(StatusMessage::NOT_FOUND)
            .header("Content-Type", "text/plain");

          match path {
            "/" => hello_world(rsp),
            _ => delay(rsp),
          }
          .and_then(move |rsp| rsp.write(writer))
        })
        .map_err(|e| println!("Error in http reading; err={:?}", e))
        .map(|_| ());

      // spawn each connection
      runtime::spawn(conn)
    });

  println!("Server is listening on {}", addr);
  runtime::run(server);
}

pub fn hello_world(rsp: Response) -> Box<Future<Item = Response, Error = std::io::Error>> {
  // you can actually map all futures and return Ok result
  let hello = futures::future::ok(rsp.status(StatusMessage::OK).body("Hello world"));

  Box::new(hello)
}

pub fn delay(rsp: Response) -> Box<Future<Item = Response, Error = std::io::Error>> {
  // delay example
  let when = Instant::now() + Duration::from_millis(2000);

  let delay = Delay::new(when)
    .map_err(|e| panic!("delay errored; err={:?}", e))
    .and_then(move |_| {
      Ok(
        rsp
          .status(StatusMessage::OK)
          .body("Got in delay page 2000ms"),
      )
    });

  Box::new(delay)
}

// pub fn not_found(rsp: Response) -> Box<Future<Item = Response, Error = std::io::Error>> {
//   let at404 = futures::future::ok(rsp.status(status::NOT_FOUND).body("Could not find"));

//   Box::new(at404)
// }
