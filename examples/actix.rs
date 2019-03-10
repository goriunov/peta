extern crate actix;
extern crate actix_web;

use actix_web::{server, App, HttpRequest};

fn index(_req: &HttpRequest) -> &'static str {
  "Hello world!"
}

fn main() {
  let sys = actix::System::new("hello-world");

  server::new(|| App::new().resource("/", |r| r.f(index)))
    .workers(1)
    .bind("127.0.0.1:3000")
    .unwrap()
    .start();

  println!("Started http server: 127.0.0.1:3000");
  sys.run();
}
