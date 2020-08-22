use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io;
use std::convert::TryInto;
use crate::jack_trip_header::*;
use crate::jack_notifications::Notifications;

mod jack_trip_header;
mod jack_notifications;

fn get_current_timestamp() -> [u8; 8]{
  (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64).to_le_bytes()
}

fn verify_connection_params(s: JackTripHeader, sample_rate: usize) {
  assert!(s.sampling_rate.as_numeric() == sample_rate);
  assert!(s.bit_resolution == 16);
  assert!(s.num_channels == 1);
  assert!(s.buffer_size == 128);
}

fn jack_test() -> std::io::Result<()> {
  let (client_receive, _status) =
    jack::Client::new("madwort_rust_trip_receive", jack::ClientOptions::NO_START_SERVER).unwrap();
  let (client_send, _status) =
    jack::Client::new("madwort_rust_trip_send", jack::ClientOptions::NO_START_SERVER).unwrap();

  // "Receive" takes audio data from the network and sends it to the local jack server
  // Therefore it is an AudioOut port from the perspective of this program
  let mut receive_a = client_receive
      .register_port("rust_receive_l", jack::AudioOut::default())
      .unwrap();
  let send_a = client_send
      .register_port("rust_send_l", jack::AudioIn::default())
      .unwrap();

  let socket_receive = UdpSocket::bind("127.0.0.1:34254")?;
  let socket_send = socket_receive.try_clone()?;

  // Receives a single datagram message on the socket. If `buf` is too small to hold
  // the message, it will be cut off.
  let mut buf = [0u8; 528];

  // get the first packet, so that we can check some params
  let (_amt, src) = socket_receive.recv_from(&mut buf)?;
  let s: JackTripHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _)};
  verify_connection_params(s, client_receive.sample_rate());

  let mut outgoing_buf = [0u8; 528];
  let mut outgoing_sequence_number = 0u16;

  // Mirror source data back to itself for now
  let mut timestamp_bytes = get_current_timestamp();

  outgoing_buf[0..8].copy_from_slice(&timestamp_bytes);
  outgoing_buf[8..10].copy_from_slice(&outgoing_sequence_number.to_le_bytes());
  // Duplicate all other data from the input packet
  outgoing_buf[10..16].copy_from_slice(&buf[10..16]);
  socket_send.send_to(&outgoing_buf, &src).unwrap();

  // Create some temp vars to use in the process_callback
  let mut temp_audio_data = [0u8; 2];
  let mut count = 0;

  let process_callback_receive = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
      // TODO: xrun when nothing to read from buf! Fixme!

      // TODO: can we allocate this outside the process_callback?
      let receive_a_p = receive_a.as_mut_slice(ps);
      // let receive_b_p = receive_b.as_mut_slice(ps);
      socket_receive.recv_from(&mut buf).unwrap();

      // Debug output
      let s: JackTripHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _)};
      println!("Input: {}", s);

      // TODO: get rid of these ugly count vars!!! OMG!
      count = 16;
      for v in receive_a_p.iter_mut() {
        temp_audio_data = buf[count..count+2].try_into().unwrap();
        *v = i16::from_le_bytes(temp_audio_data) as f32 / 32768.0;
        count = count + 2;
      }
      jack::Control::Continue
  };

  let process_callback_send = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
      let send_a_p = send_a.as_slice(ps);
      // // let send_b_p = send_b.as_slice(ps);
      count = 16;
      for v in send_a_p.iter() {
        temp_audio_data = (((*v) * 32768.0) as i16).to_le_bytes();
        outgoing_buf[count..count+2].clone_from_slice(&temp_audio_data);
        count = count + 2;
      }
      // send_a_p.clone_from_slice(output_packet.data);
      // send_b_p.clone_from_slice(&receive_b_p);

      timestamp_bytes = get_current_timestamp();

      outgoing_sequence_number = outgoing_sequence_number + 1;
      outgoing_buf[0..8].copy_from_slice(&timestamp_bytes);
      outgoing_buf[8..10].copy_from_slice(&outgoing_sequence_number.to_le_bytes());

      socket_send.send_to(&outgoing_buf, &src).unwrap();
      jack::Control::Continue
  };

  let process_receive = jack::ClosureProcessHandler::new(process_callback_receive);
  let process_send = jack::ClosureProcessHandler::new(process_callback_send);

  // Activate the client, which starts the processing.
  let active_client_receive = client_receive.activate_async(Notifications, process_receive).unwrap();
  let active_client_send = client_send.activate_async(Notifications, process_send).unwrap();

  // Wait for user input to quit
  println!("Press enter/return to quit...");
  let mut user_input = String::new();
  io::stdin().read_line(&mut user_input).ok();

  active_client_receive.deactivate().unwrap();
  active_client_send.deactivate().unwrap();
  Ok(())
}

fn main() -> std::io::Result<()> {
    jack_test()
}
