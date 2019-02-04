use bytes::{BufMut, BytesMut};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::prelude::*;

use peta::reader;

use std::time::{Duration, Instant};
use tokio::timer::Delay;

type Writer = tokio::io::WriteHalf<tokio::net::TcpStream>;
type ResponseFut = Box<dyn Future<Item = (Writer, Home), Error = ()> + Send + Sync>;

struct Home {
  pub req: Option<peta::request::Request>,
  pub res: peta::response::Response,
}

impl Home {
  pub fn on_data(mut self, writer: Writer, data: BytesMut, is_last: bool) -> ResponseFut {
    if is_last {
      println!("Going to write");
      // self.res.body_str("Socket is completed");

      return Box::new(
        self
          .res
          .write_chunk(writer, false)
          .and_then(move |writer| {
            self.res.body_str("Second part of the message is here");
            self
              .res
              .write_chunk(writer, false)
              .map(|writer| (writer, self))
          })
          .and_then(move |(writer, mut val)| {
            let when = Instant::now() + Duration::from_millis(2000);

            Delay::new(when)
              .map_err(|e| panic!("Delay errored; err={:?}", e))
              .and_then(move |_| {
                // let mut res = Response::new();
                // res.status(status::OK);
                // res.body_str("Hello world!");
                val.res.body_str("Third part of the message is here");
                val
                  .res
                  .write_chunk(writer, true)
                  .map(|writer| (writer, val))
                // Ok(res)
              })
          })
          .map_err(|e| println!("Error while writing {}", e)),
      );
    }

    // read next part of data
    Box::new(futures::future::ok((writer, self)))
  }

  pub fn on_request(mut self, writer: Writer, req: peta::request::Request) -> ResponseFut {
    self.res.completed = false;
    self.req = Some(req);
    self.res.status("200 OK");
    self.res.body_str("First part of the message");

    // return Box::new(
    //   self
    //     .res
    //     .write(writer)
    //     .map_err(|e| println!("Error while writing {}", e))
    //     .map(|writer| (writer, self)),
    // );
    Box::new(futures::future::ok((writer, self)))
  }
}

fn main() {
  let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
  let addr = "127.0.0.1:3000".parse().unwrap();

  // router.get("/hello/world");
  // router.get("/hello", || Home {
  //   req: None,
  //   res: peta::response::Response::new(),
  // });

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
        .fold(
          (write, best_req),
          move |(write, best_req), state| match state {
            reader::ReturnType::Data(data, is_last) => {
              if !best_req.res.is_completed() {
                return best_req.on_data(write, data, is_last);
              }
              return Box::new(futures::future::ok((write, best_req)));
            }
            reader::ReturnType::Request(req) => {
              return best_req.on_request(write, req);
            }
          },
        )
        // connection is closed
        .map(|item| ());

      tokio::runtime::current_thread::spawn(reader);
      Ok(())
    });

  runtime.spawn(server);
  runtime.run().unwrap();
}
