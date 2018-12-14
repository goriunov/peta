use http::*;

pub fn generate_response(res: Response<String>) -> String {
  // String::from(
  //     "HTTP/1.1 404
  // server: Ultra
  // content-type: text/plain
  // content-length: 9

  // My string",
  //   )

  let mut rsp = "HTTP/1.1 ".to_owned() + res.status().as_str() + "\r\n";

  // handler all other headers
  let headers = res.headers();

  for (key, value) in headers.iter() {
    rsp = rsp + key.as_str() + ": " + value.to_str().unwrap() + "\r\n";
  }

  let body = res.body();
  rsp = rsp + "content-length: " + format!("{}", body.len()).as_str() + "\r\n\r\n" + body;

  // println!("{}", rsp);
  rsp
}
