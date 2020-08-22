use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::io;
use crate::jack_trip_header::*;

mod jack_trip_header;
mod udp;


fn udp_listen() -> std::io::Result<()> {
  {
    let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
    // Receives a single datagram message on the socket. If `buf` is too small to hold
    // the message, it will be cut off.

    // Current static calculations:
    // header size = 64+16+16+8+8+8+8 = 128
    // jack frame size = 16*256 = 4096
    // therefore buffer size is (4096+128)/8 => u8 array length 528

    let mut buf = [0; 528];
    // output the connection details from the first packet
    socket.recv_from(&mut buf)?;
    let s: JackTripHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _)};
    println!("{}", s);

    while true {

      let (amt, src) = socket.recv_from(&mut buf)?;

      // println!("{:?}", buf);
      // println!("amt {:?}", amt);
      // println!("src {:?}", src);

      // ==v1
      let s: JackTripHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _)};
      println!("Struct: {}", s);

      match SystemTime::now().duration_since(UNIX_EPOCH) {
          Ok(elapsed) => {
              // it prints '2'
              println!("Time: {:?}, {:?}", s.time_stamp, elapsed);
              // println!("{}", elapsed.as_secs());
          }
          Err(e) => {
              // an error occurred!
              println!("Error: {:?}", e);
          }
      }

      // Redeclare `buf` as slice of the received data and send reverse data back to origin.
      let buf = &mut buf[..amt];
      // if we don't reverse it, jacktrip client accepts it & sends more!
      // buf.reverse();
      socket.send_to(buf, &src)?;
    }

  } // the socket is closed here
  Ok(())
}

fn jack_test() -> std::io::Result<()> {
  let (client, _status) = 
    jack::Client::new("madwort_rust_trip", jack::ClientOptions::NO_START_SERVER).unwrap();

  // "Receive" takes audio data from the network and sends it to the local jack server
  // Therefore it is an AudioOut port from the perspective of this program
  let mut receive_a = client
      .register_port("rust_receive_l", jack::AudioOut::default())
      .unwrap();
  // let receive_b = client
  //     .register_port("rust_receive_r", jack::AudioIn::default())
  //     .unwrap();
  let send_a = client
      .register_port("rust_send_l", jack::AudioIn::default())
      .unwrap();
  // let mut send_b = client
  //     .register_port("rust_send_r", jack::AudioOut::default())
  //     .unwrap();

  let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
  // Receives a single datagram message on the socket. If `buf` is too small to hold
  // the message, it will be cut off.
  let mut buf = [0; 528];

  // get the first packet, so that we can check some params
  socket.recv_from(&mut buf)?;
  let s: JackTripHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _)};
  assert!(s.sampling_rate.as_numeric() == client.sample_rate());
  assert!(s.bit_resolution == 16);
  assert!(s.num_channels == 1);
  assert!(s.buffer_size == 128);

  let sample_rate = client.sample_rate();
  let frame_t = 1.0 / sample_rate as f64;
  let mut output_sequence_number = 0;

  // TODO: don't hardcode these vars!!
  let mut output_packet: JackTripHeader = JackTripHeader {
    time_stamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64,
    sequence_number: output_sequence_number,
    buffer_size: 128,
    sampling_rate: SamplingRateT::SR48,
    bit_resolution: 16,
    num_channels: 1,
    connection_mode: 0,
    data: [0; 256],
  };

  let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
    // xrun when nothing to read from buf!

      let receive_a_p = receive_a.as_mut_slice(ps);
      // let receive_b_p = receive_b.as_mut_slice(ps);
      let (amt, src) = socket.recv_from(&mut buf).unwrap();
      let s: JackTripHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _)};
      println!("Input: {}", s);

      // TODO: get rid of these ugly count vars!!! OMG!
      let mut count = 0;
      for v in receive_a_p.iter_mut() {
        *v = s.get_jack_data(count);
        count = count + 1;
      }

      let send_a_p = send_a.as_slice(ps);
      // let send_b_p = send_b.as_slice(ps);
      count = 0;
      for v in send_a_p.iter() {
        count = count + 1;
      }
      // send_a_p.clone_from_slice(output_packet.data);
      // send_b_p.clone_from_slice(&receive_b_p);
      output_sequence_number = output_sequence_number + 1;
      output_packet.time_stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64;
      output_packet.sequence_number = output_sequence_number;
      println!("Output: {}", output_packet);
      let buf = &mut buf[..amt];
      // if we don't reverse it, jacktrip client accepts it & sends more!
      // buf.reverse();
      socket.send_to(buf, &src);
      jack::Control::Continue
  };
  let process = jack::ClosureProcessHandler::new(process_callback);

  // Activate the client, which starts the processing.
  let active_client = client.activate_async(Notifications, process).unwrap();

  // Wait for user input to quit
  println!("Press enter/return to quit...");
  let mut user_input = String::new();
  io::stdin().read_line(&mut user_input).ok();

  active_client.deactivate().unwrap();
  Ok(())
}

fn main() -> std::io::Result<()> {
    jack_test();
    // udp_listen();
    Ok(())
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        println!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
  }
    