use super::*;

pub enum Status {
  Chunk(BytesMut),
  Last,
  NotEnoughData,
}

pub struct Chunk {}

impl Chunk {
  // this function modifies array
  pub fn parse(buffer: &mut BytesMut) -> Result<Status, std::io::Error> {
    // TODO: We need to change usize to u64
    const RADIX: usize = 16;
    let mut size = 0;
    let mut count = 0;
    let mut pos = 0;
    let mut last = false;

    let mut bytes = buffer.iter();

    dbg!(&buffer);

    loop {
      // TODO: need to handle unwrap properly
      let byte = bytes.next().unwrap();
      pos += 1;
      match byte {
        b'0' => {
          last = true;
        }
        b'1'...b'9' => {
          // TODO: we should mark if value is actual
          last = false;
          count += 1;
          size *= RADIX;
          size += (byte - b'0') as usize;
        }
        b'a'...b'f' => {
          last = false;
          count += 1;
          size *= RADIX;
          size += (byte + 10 - b'a') as usize;
        }
        b'A'...b'F' => {
          last = false;
          count += 1;
          size *= RADIX as usize;
          size += (byte + 10 - b'A') as usize;
        }
        b'\r' => {
          // TODO: need to handle unwrap properly
          let byte = bytes.next().unwrap();
          pos += 1;
          match byte {
            b'\n' => {
              break;
            }
            _ => {
              // TODO: add errors
              break;
            }
          }
        }
        _ => {
          // TODO: add errors
          break;
        }
      }
    }

    // we have gotten last chunk
    if last {
      dbg!("The last");
      buffer.advance(5);
      return Ok(Status::Last);
    }

    // TODO: if length is not enough return error
    if buffer.len() - pos < size {
      // TODO: we need to return not ready
      return Ok(Status::NotEnoughData);
    }

    // remove length data from buffer
    buffer.advance(pos + 2);
    Ok(Status::Chunk(buffer.split_to(size)))

    // handle actual split and return bytes buffer
  }
}
