use bytes::{BufMut, BytesMut};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::prelude::*;

use peta::reader;

use std::time::{Duration, Instant};
use tokio::timer::Delay;

struct Home {
  pub req: Option<peta::request::Request>,
  pub res: peta::response::Response,
}

impl Home {
  pub fn on_data(mut self, writer: Writer, data: BytesMut, is_last: bool) -> ResponseFut {
    if is_last {
      self.res.body_str("Socket is completed");
      return Box::new(
        self
          .res
          .write(writer)
          .map_err(|e| println!("Global Error is: {}", e))
          .map(|writer| (writer, self)),
      );
    }

    // read next part of data
    Box::new(futures::future::ok((writer, self)))
  }

  pub fn on_request(mut self, writer: Writer, req: peta::request::Request) -> ResponseFut {
    self.req = Some(req);
    self.res.status("200 OK");

    Box::new(futures::future::ok((writer, self)))
  }
}

type Writer = tokio::io::WriteHalf<tokio::net::TcpStream>;

type ResponseFut = Box<dyn Future<Item = (Writer, Home), Error = ()> + Send + Sync>;

// type ResponseFut = Box<
//   dyn Future<Item = (tokio::io::WriteHalf<tokio::net::TcpStream>, Home), Error = ()> + Send + Sync,
// >;

fn main() {
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  // router.get("/hello");

  let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

  let server = listener
    .incoming()
    .map_err(|e| eprintln!("accept failed = {:?}", e))
    .for_each(move |sock| {
      let (read, write) = sock.split();
      let best_req = Home {
        req: None,
        res: peta::response::Response::new(),
      };

      let reader = reader::Reader::new(read)
        .map_err(|e| println!("Global Error is: {}", e))
        .fold((write, best_req), move |(write, best_req), state| {
          match state {
            reader::ReturnType::Data(data, is_last) => {
              return best_req.on_data(write, data, is_last);
            }
            reader::ReturnType::Request(req) => {
              return best_req.on_request(write, req);
            }
          }

          // continue looping
          // return Box::new(futures::future::ok((write, best_req)));

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
