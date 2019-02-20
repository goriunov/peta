use tokio::net::TcpListener;
use tokio::prelude::*;

use peta;

fn main() {
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");
  // peta::hello()

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(move |sock| {
      let (read, write) = sock.split();
      let reader = peta::reader::Reader::new(read)
        .map_err(|e| println!("{}", e))
        .map(|_| {
          println!("Socket closed");
          ()
        });

      tokio::runtime::current_thread::spawn(reader);
      Ok(())
    });

  runtime.spawn(server);
  runtime.run().unwrap();
}
