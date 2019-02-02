use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::prelude::*;

use peta::reader;

use std::time::{Duration, Instant};
use tokio::timer::Delay;

struct BestRequest {}

impl BestRequest {
  pub fn on_data(self) {}
  pub fn on_request(self) {}

  // pub fn change(&mut self) {
  //   self.hello = false;
  // }

  // pub fn exec(self) -> ResponseFutFunc {
  //   let when = Instant::now() + Duration::from_millis(10000);

  //   let delay = Delay::new(when)
  //     .map_err(|e| panic!("Delay errored; err={:?}", e))
  //     .and_then(|_| {
  //       Ok((
  //         String::from("HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n"),
  //         self,
  //       ))
  //     });

  //   Box::new(delay)
  //   // Box::new(futures::future::ok((
  //   //   String::from("HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n"),
  //   //   self,
  //   // )))
  // }
}

pub type ResponseFutFunc = Box<dyn Future<Item = (String, BestRequest), Error = ()> + Send + Sync>;

pub type ResponseFut =
  Box<dyn Future<Item = tokio::io::WriteHalf<tokio::net::TcpStream>, Error = ()> + Send + Sync>;

fn main() {
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(move |sock| {
      let (read, write) = sock.split();
      let item = 0;

      // let req = BestRequest { hello: false };

      // let cb = || {
      //   let index = 0;
      //   println!("Index {}", index);

      //   // return Box::new(futures::future::ok(write).and_then(|a| Ok(a)));
      // };

      // let cb = Arc::new(cb).clone();

      // let request = {
      //   on_chunk: || {}
      // }

      let reader = reader::Reader::new(read)
        .map_err(|e| println!("Global Error is: {}", e))
        .fold(write, move |write, state| -> ResponseFut {
          let mut is_ready = false;

          match state {
            reader::ReturnType::Data(data, is_last) => {
              // request is completed
              if is_last {
                return Box::new(
                  tokio::io::write_all(write, &"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n"[..])
                    .map_err(|e| println!("Global Error is: {}", e))
                    .and_then(|(a, b)| Ok(a))
                    .map(|a| a),
                );
                // dbg!(data);
              }
            }
            reader::ReturnType::Request(req) => {
              // dbg!(&req.data);
              is_ready = true;
              // futures::future::ok(write)
            }
          };

          return Box::new(futures::future::ok(write));
          // println!("Hit here");
          // if is_ready {
          // return Box::new(
          // return Box::new(
          //   tokio::io::write_all(write, &"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n"[..])
          //     .map_err(|e| println!("Global Error is: {}", e))
          //     .and_then(|(a, b)| Ok(a))
          //     .map(|a| a),
          // );
          // );
          // }
          // togther.1.change();
          // (cb)(&mut write)

          // Box::new(futures::future::ok(togther).and_then(|a| Ok(a)))

          // futures::future::ok(item)
          // let mut when = Instant::now() + Duration::from_millis(2000);

          // if item == 0 {
          //   println!("In 0");
          //   when = Instant::now() + Duration::from_millis(10000);
          // }

          // if item == 1 {
          //   println!("In 1");
          //   when = Instant::now() + Duration::from_millis(100);
          // }

          // if item == 2 {
          //   println!("In 2");
          //   when = Instant::now() + Duration::from_millis(1);
          // }

          // let delay = Delay::new(when)
          //   .map_err(|e| panic!("Delay errored; err={:?}", e))
          //   .and_then(move |_| Ok(item + 1));
          // delay
        })
        .map(|item| {
          // println!("Completed {}", item);
          ()
        });

      // let future = reader::Reader::new(read)
      //   .on_headers(|state, data| {})
      //   .on_body(|state, chunk| {
      //     if chunk.is_last() {
      //       // do some stuff
      //     }

      //     return state;
      //   })
      //   .complete(|state| {})
      //   .build();

      // future.fold(State::new(write), ||)

      // router.find()

      // let reader = reader::Reader::new(read)
      //   .map_err(|e| println!("Global Error is: {}", e))
      //   .fold(write, move |writer, req| {
      //     if (req.is_headers()) {
      //       // write.
      //     }
      //     // you can get all headers and the rest of the things here
      //     // req.lock().d
      //     // let req = req.lock().unwrap();
      //     // println!("{:#?}", req.content.as_ref().unwrap().data);
      //     // req
      //     //   .on_data
      //     //   .map_err(|e| println!("Error on data: {}", e))
      //     //   .fold(writer, |writer, data| {
      //     //     // data.is_chunk
      //     //     // process data or send some resposne
      //     //     // how to await future completion
      //     //     Ok(writer)
      //     //   })
      //     //   .and_then(|writer| {
      //     //     // data reading has ended
      //     //     tokio::io::write_all(writer, &"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n"[..])
      //     //       .map_err(|e| println!("Global Error is"))
      //     //   })
      //     // .and_then(|(a, b)| Ok(a))
      //     // .map(|(a, b)| {
      //     //   // request is completed
      //     //   // writer
      //     // })
      //     // println!("{:#?}", req.content.unwrap().data);
      //     // println!("{:#?}", req.content.unwrap().headers);

      //     // if req.is_chunked() {

      //     // }

      //     //
      //     //
      //     //
      //     // Write data to the socket
      //     tokio::io::write_all(writer, &"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n"[..])
      //       .map_err(|e| println!("Global Error is: {}", e))
      //       .and_then(|(a, b)| Ok(a))
      //   })
      //   .map(|_| ());

      tokio::runtime::current_thread::spawn(reader);
      Ok(())
    });

  runtime.spawn(server);
  runtime.run().unwrap();
}
