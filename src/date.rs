// Thanks to https://github.com/pyfisch/httpdate/
// generate and cache time

use crate::response::push;

use bytes::BytesMut;
use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH};

struct Time {
  bytes: [u8; 37],
  time: SystemTime,
}

impl Time {
  fn new() -> Time {
    let now = SystemTime::now();
    Time {
      bytes: generate_date_header(&now),
      time: now,
    }
  }
}

thread_local!(static TIME_CACHE: RefCell<Time> = RefCell::new(Time::new()));

pub fn set_date_header(buf: &mut BytesMut) {
  TIME_CACHE.with(|cache| {
    let mut cache = cache.borrow_mut();

    match cache.time.elapsed() {
      Ok(elapsed) => {
        if elapsed.as_secs() >= 1 {
          let now = SystemTime::now();
          cache.bytes = generate_date_header(&now);
          cache.time = now;
        }
      }
      Err(e) => {
        eprint!("Could not get elapsed: {}", e);
      }
    };

    push(buf, &cache.bytes);
  });
}

fn generate_date_header(v: &SystemTime) -> [u8; 37] {
  let secs_since_epoch = v
    .duration_since(UNIX_EPOCH)
    .expect("all times should be after the epoch")
    .as_secs();

  if secs_since_epoch >= 253402300800 {
    // year 9999
    panic!("date must be before year 9999");
  }

  /* 2000-03-01 (mod 400 year, immediately after feb29 */
  const LEAPOCH: i64 = 11017;
  const DAYS_PER_400Y: i64 = 365 * 400 + 97;
  const DAYS_PER_100Y: i64 = 365 * 100 + 24;
  const DAYS_PER_4Y: i64 = 365 * 4 + 1;

  let days = (secs_since_epoch / 86400) as i64 - LEAPOCH;
  let secs_of_day = secs_since_epoch % 86400;

  let mut qc_cycles = days / DAYS_PER_400Y;
  let mut remdays = days % DAYS_PER_400Y;

  if remdays < 0 {
    remdays += DAYS_PER_400Y;
    qc_cycles -= 1;
  }

  let mut c_cycles = remdays / DAYS_PER_100Y;
  if c_cycles == 4 {
    c_cycles -= 1;
  }
  remdays -= c_cycles * DAYS_PER_100Y;

  let mut q_cycles = remdays / DAYS_PER_4Y;
  if q_cycles == 25 {
    q_cycles -= 1;
  }
  remdays -= q_cycles * DAYS_PER_4Y;

  let mut remyears = remdays / 365;
  if remyears == 4 {
    remyears -= 1;
  }
  remdays -= remyears * 365;

  let mut year = 2000 + remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

  let months = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];
  let mut mon = 0;
  for mon_len in months.iter() {
    mon += 1;
    if remdays < *mon_len {
      break;
    }
    remdays -= *mon_len;
  }
  let mday = remdays + 1;
  let mon = if mon + 2 > 12 {
    year += 1;
    mon - 10
  } else {
    mon + 2
  };

  let mut wday = (3 + days) % 7;
  if wday <= 0 {
    wday += 7
  };

  let sec = (secs_of_day % 60) as u8;
  let min = ((secs_of_day % 3600) / 60) as u8;
  let hour = (secs_of_day / 3600) as u8;
  let day = mday as u8;
  let year = year as u16;

  let wday = match wday {
    1 => b"Mon",
    2 => b"Tue",
    3 => b"Wed",
    4 => b"Thu",
    5 => b"Fri",
    6 => b"Sat",
    7 => b"Sun",
    _ => unreachable!(),
  };

  let mon = match mon {
    1 => b"Jan",
    2 => b"Feb",
    3 => b"Mar",
    4 => b"Apr",
    5 => b"May",
    6 => b"Jun",
    7 => b"Jul",
    8 => b"Aug",
    9 => b"Sep",
    10 => b"Oct",
    11 => b"Nov",
    12 => b"Dec",
    _ => unreachable!(),
  };

  let mut buf: [u8; 37] = [
    // Too long to write as: b"date: Thu, 01 Jan 1970 00:00:00 GMT/n/r"
    b'd', b'a', b't', b'e', b':', b' ', b' ', b' ', b' ', b',', b' ', b'0', b'0', b' ', b' ', b' ',
    b' ', b' ', b'0', b'0', b'0', b'0', b' ', b'0', b'0', b':', b'0', b'0', b':', b'0', b'0', b' ',
    b'G', b'M', b'T', b'\r', b'\n',
  ];

  buf[6] = wday[0];
  buf[7] = wday[1];
  buf[8] = wday[2];
  buf[11] = b'0' + (day / 10) as u8;
  buf[12] = b'0' + (day % 10) as u8;
  buf[14] = mon[0];
  buf[15] = mon[1];
  buf[16] = mon[2];
  buf[18] = b'0' + (year / 1000) as u8;
  buf[19] = b'0' + (year / 100 % 10) as u8;
  buf[20] = b'0' + (year / 10 % 10) as u8;
  buf[21] = b'0' + (year % 10) as u8;
  buf[23] = b'0' + (hour / 10) as u8;
  buf[24] = b'0' + (hour % 10) as u8;
  buf[26] = b'0' + (min / 10) as u8;
  buf[27] = b'0' + (min % 10) as u8;
  buf[29] = b'0' + (sec / 10) as u8;
  buf[30] = b'0' + (sec % 10) as u8;

  buf
}
