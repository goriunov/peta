use tokio::net::TcpListener;
use tokio::prelude::*;

use peta;

fn main() {
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");
  // peta::hello()

  let mut router = peta::router::Router::new();
  router.get("/", |(mut req, res)| {
    // dbg!("Got in here");
    req.on_data(|(mut req, res)| {
      //   //   // handle rest
      // dbg!(req.data().take());
      // if req.is_last() {
      //   // let data = req.data().take();
      //   // dbg!("The last one");
      //   return Box::new(
      //     res
      //       .write("Hello world :)".as_bytes())
      //       .map(|res| ((req, res))),
      //   );
      // }

      // req.data().take();

      Box::new(res.write("Hello world".as_bytes()).map(|res| ((req, res))))
      // Box::new(futures::future::ok((req, res)))
    });

    // Box::new(res.write("Hello world".as_bytes()).map(|res| ((req, res))))

    return Box::new(futures::future::ok((req, res)));
  });

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(move |sock| {
      // sock.shutdown(std::net::Shutdown::Both);
      // sock.set_keepalive(Some(std::time::Duration::from_secs(1)));
      // sock
      // sock.set_timeout(Duration::from_secs(5));
      let reader = peta::reader::Reader::new(sock.split(), &router)
        .map_err(|e| eprintln!("Error {}", e))
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
