use crate::writer;

use tokio::net::TcpStream;

// need to optimize response generation
pub struct Response {
  body: Vec<u8>,
  header: Vec<u8>,
  status: Vec<u8>,
  version: Vec<u8>,
  delimiter: [u8; 2],
}

impl Response {
  pub fn new() -> Response {
    Response {
      version: Vec::from("HTTP/1.1 "),
      status: Vec::new(),
      header: Vec::new(),
      body: Vec::new(),
      delimiter: *b"\r\n",
    }
  }

  pub fn status(mut self, status: &str) -> Response {
    let status = status.as_bytes();
    self.status.reserve(status.len() + 4);
    self.status.extend_from_slice(status);
    self.status.extend_from_slice(&self.delimiter);

    self
  }

  pub fn header(mut self, header: &str) -> Response {
    let header = header.as_bytes();
    self.header.reserve(header.len() + 4);
    self.header.extend_from_slice(header);
    self.header.extend_from_slice(&self.delimiter);

    self
  }

  pub fn body(mut self, body: &str) -> Response {
    let len = body.len();
    let cl_header = format!("Content-Length: {}\r\n\r\n", len);
    let cl_header_bytes = cl_header.as_bytes();

    self.body.reserve(cl_header_bytes.len() + 8 + len);
    self.body.extend_from_slice(cl_header_bytes);

    let body = body.as_bytes();
    self.body.extend_from_slice(body);

    self
  }

  pub fn body_vec(mut self, body: Vec<u8>) -> Response {
    let len = body.len();
    let cl_header = format!("Content-Length: {}\r\n\r\n", len);
    let cl_header_bytes = cl_header.as_bytes();
    self.body.reserve(cl_header_bytes.len() + 8 + len);
    self.body.extend_from_slice(cl_header_bytes);

    self.body.extend_from_slice(&body[..]);

    self
  }

  pub fn write(
    mut self,
    writer: tokio::io::WriteHalf<TcpStream>,
  ) -> writer::WriteAll<tokio::io::WriteHalf<TcpStream>> {
    self.version.extend(self.status);
    self.version.extend(self.header);
    self.version.extend(self.body);

    writer::write_all(writer, self.version)
  }
}

// Old from response gen
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
