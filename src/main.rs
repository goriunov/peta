use simple_http::prelude::*;
use simple_http::{reader::HttpReader, response::Response, runtime, status, Server};

// for timer example
use tokio::prelude::*;
use tokio::timer::Delay;

use std::time::{Duration, Instant};

fn main() {
  let addr = "0.0.0.0:3000";

  let server = Server::new(&addr)
    .map_err(|e| println!("failed to accept socket; error = {:?}", e))
    .for_each(|socket| {
      let (reader, writer) = socket.split();

      let conn = HttpReader::new(reader)
        .fold(writer, |writer, req| {
          let rsp = Response::new()
            .header("Server: Ultra")
            .header("Content-Type: text/plain");

          // println!("Path: {}", req.path());
          // println!("Method: {}", req.method());

          // Delay::new(when)
          //   .map_err(|e| panic!("delay errored; err={:?}", e))
          //   .and_then(move |_| rsp.write(writer))

          // .map(move |_| {
          //   println!("Has been completed");
          //   // rsp.write(writer)
          //   // .map_err(|e| panic!("delay errored; err={:?}", e))
          //   // .map(|writer| writer)

          //   // writer
          // })

          // .map(|resp| writer)
          // .wait()

          // rsp.write(writer)

          // data

          index(rsp)
            .map_err(|e| panic!("delay errored;"))
            .and_then(move |rsp: Response| rsp.write(writer))

          // match req.path() {
          //   "/" => index(rsp),
          //   _ => not_found(rsp),
          // }
          // .write(writer)
        })
        .map_err(|e| panic!("delay errored; err={:?}", e))
        .map(|_| ());

      // spawn each connection
      runtime::spawn(conn)
    });

  println!("Server is listening on {}", addr);
  runtime::run(server);
}

pub fn index(rsp: Response) -> impl Future<Item = Response> {
  // delay example
  let when = Instant::now() + Duration::from_millis(5000);

  Delay::new(when)
    .map_err(|e| panic!("delay errored; err={:?}", e))
    .and_then(move |_| Ok(rsp.status(status::OK).body("/ got in Index function")))
  // .and_then(|_| Ok(()))
  // let when = Instant::now() + Duration::from_millis(100);

  // Delay::new(when)
  //   .map_err(|e| panic!("delay errored; err={:?}", e))
  //   .and_then(|_| Ok(rsp.status(status::OK).body("/ got in Index function")))
  //   .map(|resp| resp.write(writer))
  // runtime::spawn(time)
  // do some cool logic

  // do some request to the db
  // rsp.status(status::OK).body("/ got in Index function")
}

pub fn not_found(rsp: Response) -> Response {
  rsp.status(status::NOT_FOUND).body("Could not find")
}
