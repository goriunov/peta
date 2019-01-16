use tokio::net::TcpListener;
use tokio::prelude::*;

use peta::reader;

fn main() {
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(move |sock| {
      let (read, write) = sock.split();

      let reader = reader::Reader::new(read)
        .map_err(|e| println!("Global Error is: {}", e))
        .fold(write, move |writer, req| {
          println!("{:#?}", req.content.unwrap().data);
          // println!("{:#?}", req.content.unwrap().headers);

          tokio::io::write_all(writer, &"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n"[..])
            .map_err(|e| println!("Global Error is: {}", e))
            .and_then(|(a, b)| Ok(a))
          // future::ok(writer)
        })
        .map(|_| ());

      tokio::runtime::current_thread::spawn(reader);
      Ok(())
    });

  runtime.spawn(server);
  runtime.run().unwrap();
}
