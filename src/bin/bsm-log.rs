use chrono::Utc;
use std::{
  fs,
  io::{stdin, Write},
  thread,
  time::Duration,
};

const LOG_DIR: &str = "logs";

fn main() {
  fs::create_dir_all(LOG_DIR).unwrap();

  loop {
    let mut line = String::new();
    stdin().read_line(&mut line).unwrap();

    let now = Utc::now();
    let mut file = fs::OpenOptions::new().write(true).append(true).create(true).open(format!("{}/{}.log", LOG_DIR, now.format("%Y-%m-%d"))).unwrap();

    write!(file, "{}", line).unwrap();

    thread::sleep(Duration::from_millis(100));
  }
}
