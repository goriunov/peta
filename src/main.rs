use simple_http::prelude::*;
use simple_http::{reader::Http, response, runtime, writer, Server};
use std::string::*;

fn main() {
  let addr = "0.0.0.0:3000";

  let server = Server::new(&addr)
    .map_err(|e| println!("failed to accept socket; error = {:?}", e))
    .for_each(|socket| {
      let (reader, writer) = socket.split();

      let http_handler = Http::new(reader)
        .map_err(|e| println!("Failed on http handler; error = {:?}", e))
        .fold(writer, |writer, req| {
          // println!("{}", req.method());

          let response = Response::builder()
            .status(StatusCode::OK)
            .header("Server", "Ultra")
            .header("Content-Type", "text/plain")
            .body("{\"test\": \"hello world\"}".to_string())
            .unwrap();

          // let res = format!(
          //   "{} {}\r\n\
          //    \r\n\r\n",
          //   response.version(),
          //   response.status()
          // );
          // .to_string();

          // println!("{:?}", res);
          // .generate();

          // println!("{}", response);

          // let response = match req.uri().path() {
          //   "/" => {
          //     "HTTP/1.1 200 Ok\r\nContent-Type: text/plain\r\nContent-Length: 11\r\n\r\nHello world"
          //       .to_string()
          //   }
          //   _ => {
          //     "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nConnection: close\r\nContent-Length: 10\r\n\r\nNot found!"
          //       .to_string()
          //   }
          // };

          // let res = response::generate_response(response);

          // println!("{}", res);

          writer::write_all(writer, response).map_err(|err| eprintln!("connection error: {}", err))
        })
        .map(|_| ());

      runtime::spawn(http_handler);
      Ok(())
    });

  println!("Server is listening on {}", addr);
  runtime::run(server);
}

// // extern crate futures;
// // extern crate tokio;
// // extern crate tokio_io;

// // use tokio::net::{TcpListener, TcpStream};
// // use tokio::prelude::*;
// // // use tokio_io::{AsyncRead, AsyncWrite};
// // use futures::stream::Stream;
// // use futures::{Async, Poll};

// // #[derive(Debug)]
// // struct Socket {
// //   headers: Vec<(String, String)>,
// //   socket: TcpStream,
// //   read_buf: Vec<u8>,
// // }

// // impl Socket {
// //   fn new(socket: TcpStream) -> Socket {
// //     Socket {
// //       headers: vec![],
// //       socket,
// //       read_buf: vec![],
// //     }
// //   }
// // }

// // impl Stream for Socket {
// //   type Item = Vec<(String, String)>;
// //   type Error = std::io::Error;

// //   fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
// //     // loop {
// //     self.socket.read_buf(&mut self.read_buf).map(|async| {
// //       async.map(|_| {
// //         // println!("{:?}", self.read_buf);
// //         // println!("{}", res.is_partial());

// //         let mut headers = [httparse::EMPTY_HEADER; 16];
// //         let mut req = httparse::Request::new(&mut headers);

// //         let res = req.parse(&self.read_buf[..]).unwrap();
// //         match res {
// //           httparse::Status::Complete(amt) => {
// //             println!("Not parsial any more");
// //             self
// //               .headers
// //               .push(("path".to_string(), req.path.unwrap().to_string()));
// //             // let response =
// //             Some(self.headers.clone())
// //           }
// //           httparse::Status::Partial => Some(self.headers.clone()),
// //         }
// //       })
// //     })
// //     // }

// //     // match read {
// //     //   Ok(0) => {
// //     //     println!("Did not get anything");
// //     //     break;
// //     //   }
// //     //   Ok(val) => {
// //     //     println!("Got some message {}, {:?}", val, self.buf);
// //     //   }
// //     //   Err(e) => {
// //     //     println!("GOt an error {}", e);
// //     //     break;
// //     //   }
// //     // }
// //     // }
// //     // Ok(Async::NotReady)
// //   }
// // }

// // fn main() {
// //   let addr = "127.0.0.1:3000".parse().unwrap();

// //   let listener = TcpListener::bind(&addr).unwrap();

// //   let server = listener
// //     .incoming()
// //     .map_err(|e| println!("failed to accept socket; error = {:?}", e))
// //     .for_each(|mut socket| {
// //       let read = Socket::new(socket);
// //       let read = read
// //         .map_err(|e| println!("Got an error {}", e))
// //         .for_each(|req| {
// //           println!("{:?}", req);

// //           if req.len() > 0 {
// //             match req[0].1.as_str() {
// //               "/" => {
// //                 println!("Got main path");
// //               }
// //               _ => {
// //                 println!("lost 404");
// //               }
// //             }
// //           }

// //           Ok(())
// //         }).or_else(|_| Ok(()));
// //       //

// //       tokio::spawn(read);
// //       Ok(())
// //     });

// //   println!("Server is running on 3000");
// //   tokio::run(server);
// // }

// // extern crate futures;
// // extern crate httparse;
// // extern crate simple_http;

// use futures::prelude::*;

// fn main() {
//   // current thread run time
//   let mut runtime = simple_http::rt();

//   // need to do some thing with handler to hold it outside
//   let handler = runtime.handle();
//   let server = simple_http::Server::new("0.0.0.0:3000").for_each(move |(reader, writer)| {
//     // let read = conn.read();
//     // let write = conn.read();
//     // let another_writer = writer.clone();

//     let ft = reader
//       .fold(writer, |writer, msg| {
//         // let mut headers = [httparse::EMPTY_HEADER; 16];
//         // let mut req = httparse::Request::new(&mut headers);

//         // req.parse(&msg[..]).unwrap();

//         let amt = tokio::io::write_all(
//         writer,
//         &b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, World!"
//           [..],);

//         let amt = amt.map(|(writer, _)| {
//           // println!("Written");
//           // tokio::io::shutdown(writer);
//           // writer.shutdown();
//           // writer.shutdown().unwrap();
//           // Ok(())

//           writer
//         });

//         amt.map_err(|err| {
//           println!("In Write connection error: {}", err);
//         })

//         // println!("{:?}", req.headers);

//         // think how to handle write method
//         // conn.write()
//         // ()
//       }).map(|_| {
//         println!("Closed connection");
//         ()
//       });

//     handler.spawn(ft);
//     // we need to spawn new connection

//     // runtime::spawn(reading);
//     // we may need to call rt from inside of here
//     // handle other things
//     Ok(())
//   });

//   runtime.spawn(server);
//   runtime.run();
//   // run server in rt

//   // simple_http::Server::listen("0.0.0.0:3000".parse().unwrap());
//   // .for_each(|req, res| {
//   // });
// }
