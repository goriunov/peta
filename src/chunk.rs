use super::*;

pub enum ParseStatus {
  Chunk(BytesMut),
  LastChunk(Option<BytesMut>),
  NotEnoughData,
}

pub struct Chunk {}

// TODO: implement proper parsing with validation the end of the request
impl Chunk {
  pub fn parse(buffer: &mut BytesMut) -> Result<ParseStatus, std::io::Error> {
    let mut pos = 1; // we start with 1 as we do one additional next at the bottom
    let mut size = 0;
    let mut is_last = false;

    let mut bytes_iter = buffer.iter();

    loop {
      // TODO: handle unwrap
      let byte = bytes_iter
        .next()
        .expect("Could not unwrap bytes_iter.next()");
      pos += 1;

      match byte {
        b'0'...b'9' => {
          size *= 16;
          size += (byte - b'0') as usize;
        }

        b'a'...b'f' => {
          size *= 16;
          size += (byte + 10 - b'a') as usize;
        }
        b'A'...b'F' => {
          size *= 16;
          size += (byte + 10 - b'A') as usize;
        }
        b'\r' => {
          let byte = bytes_iter
            .next()
            .expect("Could not unwrap bytes_iter.next()");
          match byte {
            b'\n' => {
              if size == 0
                || buffer.len() - pos - size == 7 && bytes_iter.nth(size + 2).unwrap() == &b'0'
              {
                is_last = true;
              }
              break;
            }
            _ => {
              // actual error
            }
          };
        }
        _ => {
          break;
          // actual error
        }
      }
    }

    if buffer.len() < size + pos + 2 {
      return Ok(ParseStatus::NotEnoughData);
    }

    buffer.advance(pos);
    if is_last {
      if size == 0 {
        buffer.advance(2);
        return Ok(ParseStatus::LastChunk(None));
      }

      let data = buffer.split_to(size);
      buffer.advance(7);
      return Ok(ParseStatus::LastChunk(Some(data)));
    }

    let data = buffer.split_to(size);
    buffer.advance(2);

    return Ok(ParseStatus::Chunk(data));
  }

  // // this function modifies array
  // pub fn parse_1(buffer: &mut BytesMut) -> Result<Status, std::io::Error> {
  //   // TODO: We need to change usize to u64
  //   const RADIX: usize = 16;
  //   let mut size = 0;
  //   let mut count = 0;
  //   let mut pos = 0;
  //   let mut last = false;

  //   let mut bytes = buffer.iter();

  //   // we need to improve reading
  //   loop {
  //     // TODO: need to handle unwrap properly
  //     let byte = bytes.next().unwrap();
  //     pos += 1;
  //     match byte {
  //       b'0' => {
  //         last = true;
  //       }
  //       b'1'...b'9' => {
  //         // TODO: we should mark if value is actual
  //         last = false;
  //         count += 1;
  //         size *= RADIX;
  //         size += (byte - b'0') as usize;
  //       }
  //       b'a'...b'f' => {
  //         last = false;
  //         count += 1;
  //         size *= RADIX;
  //         size += (byte + 10 - b'a') as usize;
  //       }
  //       b'A'...b'F' => {
  //         last = false;
  //         count += 1;
  //         size *= RADIX as usize;
  //         size += (byte + 10 - b'A') as usize;
  //       }
  //       b'\r' => {
  //         // TODO: need to handle unwrap properly
  //         let byte = bytes.next().unwrap();
  //         pos += 1;
  //         match byte {
  //           b'\n' => {
  //             break;
  //           }
  //           _ => {
  //             // TODO: add errors
  //             break;
  //           }
  //         }
  //       }
  //       _ => {
  //         // TODO: add errors
  //         break;
  //       }
  //     }
  //   }

  //   // we have gotten last chunk
  //   if last {
  //     dbg!("The last");
  //     buffer.advance(5);
  //     return Ok(Status::Last);
  //   }

  //   // TODO: if length is not enough return error
  //   if buffer.len() < size + pos + 2 {
  //     // TODO: we need to return not ready
  //     return Ok(Status::NotEnoughData);
  //   }

  //   // remove length data from buffer
  //   buffer.advance(pos);

  //   let data = buffer.split_to(size);
  //   buffer.advance(2);
  //   Ok(Status::Chunk(data))

  //   // handle actual split and return bytes buffer
  // }
}
