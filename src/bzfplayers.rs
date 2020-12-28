use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;

const BUFFER_SIZE: usize = 1024;
const MSG_QUERY_PLAYERS: &[u8; 2] = b"qp";

fn unpack_u16(bytes: &[u8], index: usize) -> u16 {
  if bytes.len() < index * 2 + 2 {
    panic!("tried to call unpack_u16 on slice too short. len: {} index: {}", bytes.len(), index);
  }

  (bytes[index * 2] as u16 >> 8) | (bytes[index * 2 + 1] as u16)
}

fn get_response(stream: &mut TcpStream, buffer: &mut [u8], code: &[u8; 2]) {
  let cmd_data_length;
  let mut cmd_buffer = [0 as u8; 4];

  loop {
    stream.read_exact(&mut cmd_buffer).unwrap();

    if &cmd_buffer[2..4] != code {
      continue;
    }

    cmd_data_length = unpack_u16(&cmd_buffer, 0);
    stream.read_exact(&mut buffer[0..cmd_data_length as usize]).unwrap();

    break;
  }
}

fn cmd(stream: &mut TcpStream, buffer: &mut [u8], code: &[u8; 2]) {
  stream.write(&[0u8, 0u8, code[0], code[1]]).unwrap();
  get_response(stream, buffer, code);
}

pub fn get_count(port: &String) -> u16 {
  let mut stream = TcpStream::connect("0.0.0.0:".to_string() + port).unwrap();

  // send magic header
  stream.write(b"BZFLAG\r\n\r\n").unwrap();

  let mut buffer = [0u8; BUFFER_SIZE];

  // check magic and protocol version
  stream.read(&mut buffer).unwrap();
  if &buffer[0..8] != b"BZFS0221" {
    let text = from_utf8(&buffer).unwrap();
    panic!("invalid protocol version: {}", text);
  }

  cmd(&mut stream, &mut buffer, MSG_QUERY_PLAYERS);

  // unpack player count
  unpack_u16(&buffer, 1)
}
