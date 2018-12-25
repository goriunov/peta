use peta;

use tokio::net::TcpListener;
use tokio::prelude::*;

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
          res.body_str("Hello world!");

          // println!("Is working");

          res.write(write).map_err(|e| println!("{}", e))
          // let status = "Hello world";

          // let body = req.body();
          // let version = req.version();

          // println!("{:?}", body);
          // write data to the socket
          // tokio::io::write_all(write, "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
          //   .map_err(|e| println!("{}", e))
          //   .map(|(w, _)| w)
        })
        .map(|_| ());

      tokio::runtime::current_thread::spawn(reader);
      Ok(())
    });

  runtime.spawn(server);
  runtime.run();
}
