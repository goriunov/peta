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
          // let mut rsp = Response::new();
          let method = req.method();

          let rsp = Response::new()
            .status(StatusMessage::NOT_FOUND)
            .header("Content-Type", "text/plain");

          match method {
            "/" => hello_world(rsp),
            _ => hello_world(rsp),
          }
          .and_then(move |rsp| rsp.write(writer))

          // rsp.write(writer)
          // write(rsp, writer)

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
          // delay(rsp, writer).map_err(|e| panic!("delay errored;"))
          // not_found(rsp)
          // match req.path() {
          //   "/" => hello_world(rsp),
          //   "/delay" => delay(rsp),
          //   _ => not_found(rsp),
          // }
          // .and_then(move |res| res.write(writer))
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
          .body("/ got in Index function"),
      )
    });

  Box::new(delay)
}

// pub fn not_found(rsp: Response) -> Box<Future<Item = Response, Error = std::io::Error>> {
//   let at404 = futures::future::ok(rsp.status(status::NOT_FOUND).body("Could not find"));

//   Box::new(at404)
// }
