use tokio::net::TcpListener;
use tokio::prelude::*;

use peta;

fn main() {
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

  let router = peta::router::Router::new(|(mut req, mut res)| {
    res.status("200 OK");
    res.body_str("Hello world");

    req.on_data(|(mut req, res)| {
      // println!("Data is going in req");
      let data = req.data();
      dbg!(data);
      if req.is_last() {
        //   // write output to the client
        return Box::new(res.write().map(|res| ((req, res))));
      }

      // waiting for next part of the data as it was not last
      Box::new(futures::future::ok((req, res)))
    });

    return Box::new(futures::future::ok((req, res)));
    // return Box::new(res.write().map(|res| ((req, res))));
  });

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(move |sock| {
      let (read, write) = sock.split();
      let reader = peta::reader::Reader::new(read, write, &router)
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
