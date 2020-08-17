use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::io;

#[repr(C, packed)]
#[derive(Debug)]
struct JTHeader {
  timeStamp: u64, ///< Time Stamp
  seqNumber: u16, ///< Sequence Number
  bufferSize: u16, ///< Buffer Size in Samples
  samplingRate: u8, ///< Sampling Rate in JackAudioInterface::samplingRateT
  bitResolution: u8, ///< Audio Bit Resolution
  numChannels: u8, ///< Number of Channels, we assume input and outputs are the same
  connectionMode: u8,
  data: [u8; 18],
}

fn udp_listen() -> std::io::Result<()> {
  {
    let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
    // Receives a single datagram message on the socket. If `buf` is too small to hold
    // the message, it will be cut off.
    let mut buf = [0; 32];

    while true {

      let (amt, src) = socket.recv_from(&mut buf)?;

      // println!("{:?}", buf);
      // println!("amt {:?}", amt);
      // println!("src {:?}", src);

      // ==v1
      let s: JTHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _)};
      println!("Struct: {:?}", s);

      match SystemTime::now().duration_since(UNIX_EPOCH) {
          Ok(elapsed) => {
              // it prints '2'
              println!("Time: {:?}, {:?}", s.timeStamp, elapsed);
              // println!("{}", elapsed.as_secs());
          }
          Err(e) => {
              // an error occurred!
              println!("Error: {:?}", e);
          }
      }
      // attempt to read directly...
      // actually should use something like `ptr << u16`?
      // unsafe {
      //   println!("{:?}", std::ptr::slice_from_raw_parts(buf.as_ptr(), 16));
      // }

      // Redeclare `buf` as slice of the received data and send reverse data back to origin.
      let buf = &mut buf[..amt];
      // if we don't reverse it, jacktrip client accepts it & sends more!
      // buf.reverse();
      socket.send_to(buf, &src)?;
    }

  } // the socket is closed here
  Ok(())
}

fn jack_test() {
  let (client, _status) = 
    jack::Client::new("madwort_rust_trip", jack::ClientOptions::NO_START_SERVER).unwrap();
  // Register ports. They will be used in a callback that will be
  // called when new data is available.
  let receive_a = client
      .register_port("rust_receive_l", jack::AudioIn::default())
      .unwrap();
  let receive_b = client
      .register_port("rust_receive_r", jack::AudioIn::default())
      .unwrap();
  let mut send_a = client
      .register_port("rust_send_l", jack::AudioOut::default())
      .unwrap();
  let mut send_b = client
      .register_port("rust_send_r", jack::AudioOut::default())
      .unwrap();
  let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
      let send_a_p = send_a.as_mut_slice(ps);
      let send_b_p = send_b.as_mut_slice(ps);
      let receive_a_p = receive_a.as_slice(ps);
      let receive_b_p = receive_b.as_slice(ps);
      send_a_p.clone_from_slice(&receive_a_p);
      send_b_p.clone_from_slice(&receive_b_p);
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
    