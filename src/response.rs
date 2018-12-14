use crate::writer;

use tokio::net::TcpStream;
use tokio::prelude::*;

pub struct Response {
  http_response: String,
}

impl Response {
  pub fn new() -> Response {
    Response {
      http_response: String::from("HTTP/1.1 "),
    }
  }

  pub fn status(mut self, status: &str) -> Response {
    self.http_response.push_str(status);
    self.http_response.push_str("\r\n");
    self
  }

  pub fn header(mut self, header: &str) -> Response {
    self.http_response.push_str(header);
    self.http_response.push_str("\r\n");
    self
  }

  pub fn body(mut self, body: &str) -> Response {
    let cl_header = format!("Content-Length: {}\r\n\r\n", body.len());
    self.http_response.push_str(cl_header.as_str());
    self.http_response.push_str(body);
    self
  }

  pub fn write(
    self,
    writer: tokio::io::WriteHalf<TcpStream>,
  ) -> writer::WriteAll<tokio::io::WriteHalf<TcpStream>> {
    writer::write_all(writer, self.http_response.into_bytes())
  }
}

// pub fn generate_response(res: Response<String>) -> String {
//   // String::from(
//   //     "HTTP/1.1 404
//   // server: Ultra
//   // content-type: text/plain
//   // content-length: 9

//   // My string",
//   //   )

//   let mut rsp = "HTTP/1.1 ".to_owned() + res.status().as_str() + "\r\n";

//   // handler all other headers
//   let headers = res.headers();

//   for (key, value) in headers.iter() {
//     rsp = rsp + key.as_str() + ": " + value.to_str().unwrap() + "\r\n";
//   }

//   let body = res.body();
//   rsp = rsp + "content-length: " + format!("{}", body.len()).as_str() + "\r\n\r\n" + body;

//   // println!("{}", rsp);
//   rsp
// }
