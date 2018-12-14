use simple_http::prelude::*;
use simple_http::{reader::Http, response::Response, runtime, status, Server};

fn main() {
  let addr = "0.0.0.0:3000";

  let server = Server::new(&addr)
    .map_err(|e| println!("failed to accept socket; error = {:?}", e))
    .for_each(|socket| {
      let (reader, writer) = socket.split();

      let http_handler = Http::new(reader)
        .fold(writer, |writer, req| {
          // println!("{}", req.method());

          let rsp = Response::new()
            .header("Server: Ultra")
            .header("Content-Type: text/plain");

          let response = match req.uri().path() {
            "/" => rsp.status(status::OK).body("/ path"),
            "/home" => rsp.status(status::OK).body("/home path"),
            _ => rsp.status(status::NOT_FOUND).body("Could not find"),
          };

          response.write(writer)
        })
        .map_err(|e| println!("Failed on http handler; Error = {:?}", e))
        .map(|_| ());

      // spawn each connection 
      runtime::spawn(http_handler);
      Ok(())
    });

  println!("Server is listening on {}", addr);
  runtime::run(server);
}
