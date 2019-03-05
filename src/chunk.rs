use super::*;

pub enum ParseStatus {
  // is_last: bool, data: BytesMut
  Chunk(bool, BytesMut),
  NotEnoughData,
}

pub fn parse(buffer: &mut BytesMut) -> Result<ParseStatus, std::io::Error> {
  let mut error_counter = 16;
  let mut pos = 1; // we start with 1 as we do one additional pos move at the "\r" part
  let mut size = 0;
  let mut is_last = false;

  let mut bytes_iter = buffer.iter();

  let mut in_chunk_size = true;
  let mut in_ext = false;

  loop {
    let byte = bytes_iter
      .next()
      .expect("Could not unwrap bytes_iter.next()");
    pos += 1;

    if error_counter == 0 {
      return Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Invalid chunk",
      ));
    }

    match byte {
      b'0'...b'9' if in_chunk_size => {
        error_counter -= 1;
        size *= 16;
        size += (byte - b'0') as usize;
      }
      b'a'...b'f' if in_chunk_size => {
        error_counter -= 1;
        size *= 16;
        size += (byte + 10 - b'a') as usize;
      }
      b'A'...b'F' if in_chunk_size => {
        error_counter -= 1;
        size *= 16;
        size += (byte + 10 - b'A') as usize;
      }

      b'\r' => {
        let byte = bytes_iter
          .next()
          .expect("Could not unwrap bytes_iter.next()");

        match byte {
          b'\n' => {
            if size == 0 {
              is_last = true;
              break;
            }
            if buffer.len() - pos - size == 7 && bytes_iter.nth(size + 2).unwrap() == &b'0' {
              is_last = true;
            }
            break;
          }
          _ => {
            return Err(std::io::Error::new(
              std::io::ErrorKind::Other,
              "Invalid chunk",
            ));
          }
        };
      }
      b';' if !in_ext => {
        in_ext = true;
        in_chunk_size = false;
      }
      b'\t' | b' ' if !in_ext & !in_chunk_size => {}
      b'\t' | b' ' if in_chunk_size => in_chunk_size = false,
      _ if in_ext => {}
      _ => {
        return Err(std::io::Error::new(
          std::io::ErrorKind::Other,
          "Invalid chunk",
        ));
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
      return Ok(ParseStatus::Chunk(true, buffer.split_to(size)));
    }

    let data = buffer.split_to(size);
    buffer.advance(7);
    return Ok(ParseStatus::Chunk(true, data));
  }

  let data = buffer.split_to(size);
  buffer.advance(2);

  return Ok(ParseStatus::Chunk(false, data));
}
