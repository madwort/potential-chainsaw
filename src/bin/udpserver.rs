use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;
use std::convert::TryInto;

// Sampling Rates supported by JACK
// Copied from jacktrip/AudioInterface.h
// This is packed as a u8
enum SamplingRateT {
    SR22, ///<  22050 Hz
    SR32, ///<  32000 Hz
    SR44, ///<  44100 Hz
    SR48, ///<  48000 Hz
    SR88, ///<  88200 Hz
    SR96, ///<  96000 Hz
    SR192, ///< 192000 Hz
    UNDEF
}

impl fmt::Display for SamplingRateT {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      SamplingRateT::SR22  => write!(f, "22kHz"),
      SamplingRateT::SR32  => write!(f, "32kHz"),
      SamplingRateT::SR44  => write!(f, "44.1kHz"),
      SamplingRateT::SR48  => write!(f, "48kHz"),
      SamplingRateT::SR88  => write!(f, "88kHz"),
      SamplingRateT::SR96  => write!(f, "96kHz"),
      SamplingRateT::SR192 => write!(f, "192kHz"),
      SamplingRateT::UNDEF => write!(f, "UNKNOWN!")
    }
  }
}

impl From<u8> for SamplingRateT {
  fn from(item: u8) -> Self {
    match item {
      0 => SamplingRateT::SR22,
      1 => SamplingRateT::SR32,
      2 => SamplingRateT::SR44,
      3 => SamplingRateT::SR48,
      4 => SamplingRateT::SR88,
      5 => SamplingRateT::SR96,
      6 => SamplingRateT::SR192,
      _ => SamplingRateT::UNDEF
    }
  }
}

#[repr(C, packed)]
// #[derive(Debug)]
struct JackTripHeader {
  time_stamp: u64, ///< Time Stamp
  sequence_number: u16, ///< Sequence Number
  buffer_size: u16, ///< Buffer Size in Samples
  sampling_rate: SamplingRateT, ///< Sampling Rate in JackAudioInterface::samplingRateT
  bit_resolution: u8, ///< Audio Bit Resolution
  num_channels: u8, ///< Number of Channels, we assume input and outputs are the same
  connection_mode: u8,
  // assume bit res 16 (i16 elements) & max buffer size 256 (array size 256)
  data: [i16; 256], // Jack frames per period size (typically 64/128/256 etc)
}

impl JackTripHeader {
  fn jack_data(&self, index: usize) -> f32{
    if self.bit_resolution != 16 {
      panic!("We only support jacktrip packets with 16bit audio data!!");
    }
    self.data[index] as f32 / 32768.0
  }
}

impl fmt::Display for JackTripHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unsafe{
      writeln!(f, "{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n", 
        "time_stamp", self.time_stamp,
        "sequence_number", self.sequence_number,
        "buffer_size", self.buffer_size,
        "sampling_rate", self.sampling_rate,
        "bit_resolution", self.bit_resolution,
        "num_channels", self.num_channels,
        "connection_mode", self.connection_mode
      )
    }
  }
}

fn print_sample_data_from_buf_unsafe(buf: &[u8]) {
  let s: JackTripHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _)};
  println!("Buffer size u8 {}", s.buffer_size);

  for x in 0..s.buffer_size as usize {
    println!("{} ; {:?} ; {:?}", x, s.jack_data(x), s.data[x]);
  }

  // println!("Struct: {:?}", s);
}

fn print_sample_data_from_buf(buf: &[u8]) {
  let buffer_size_direct_read = u16::from_le_bytes(buf[10..12].try_into().unwrap());
  println!("Buffer size u8 {}", buffer_size_direct_read);

  for x in 0..buffer_size_direct_read as usize {
    println!("{} ; {:?}", x,
      i16::from_le_bytes(buf[((x*2)+16)..((x*2)+18)].try_into().unwrap())
    );
  }
}

fn print_sample_data_from_buf_both(buf: &[u8]) {
  let s: JackTripHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _)};
  let buffer_size_direct_read = u16::from_le_bytes(buf[10..12].try_into().unwrap());
  println!("Buffer size u8 {}", buffer_size_direct_read);

  for x in 0..buffer_size_direct_read as usize {
    println!("{} ; {:?}; {:?}", x, s.data[x],
      i16::from_le_bytes(buf[((x*2)+16)..((x*2)+18)].try_into().unwrap())
    );
  }
}

fn main() -> std::io::Result<()> {
    {
      let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
      // Receives a single datagram message on the socket. If `buf` is too small to hold
      // the message, it will be cut off.

      // Current static calculations:
      // header size = 64+16+16+8+8+8+8 = 128
      // jack frame size = 16*256 = 4096
      // therefore buffer size is (4096+128)/8 => u8 array length 528

      let mut buf = [0u8; 528];

      // output the connection details from the first packet
      let (_amt, src) = socket.recv_from(&mut buf)?;

      println!("Read the buffer using unsafe case");
      let s: JackTripHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _)};
      println!("{}", s);

      println!("Read the buffer using try_into / from_le_bytes etc");
      println!("{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n",
        "time_stamp", u64::from_le_bytes(buf[0..8].try_into().unwrap()),
        "sequence_number", u16::from_le_bytes(buf[8..10].try_into().unwrap()),
        "buffer_size", u16::from_le_bytes(buf[10..12].try_into().unwrap()),
        "sampling_rate", SamplingRateT::from(buf[12]),
        "bit_resolution", buf[13],
        "num_channels", buf[14],
        "connection_mode", buf[15]
      );

      // Send our first outbound packet, reflecting details back at them
      let mut outgoing_buf = [0u8; 528];
      let mut outgoing_sequence_number = 0u16;

      // Mirror source data back to itself for now
      let mut timestamp_bytes =
        (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64).to_le_bytes();
      // TODO: there *MUST* be a better way of doing this!
      for x in 0..8 {
        outgoing_buf[x] = timestamp_bytes[x];
      }
      outgoing_buf[8] = outgoing_sequence_number.to_le_bytes()[0];
      outgoing_buf[9] = outgoing_sequence_number.to_le_bytes()[1];
      outgoing_buf[10] = 128u16.to_le_bytes()[0];
      outgoing_buf[11] = 128u16.to_le_bytes()[1];
      outgoing_buf[12] = buf[12];
      outgoing_buf[13] = buf[13];
      outgoing_buf[14] = buf[14];
      outgoing_buf[15] = buf[15];
      let t: JackTripHeader = unsafe { std::ptr::read(outgoing_buf.as_ptr() as *const _)};
      println!("{}", t);
      socket.send_to(&outgoing_buf, &src)?;

      // return Ok(());

      while true {

        let (amt, src) = socket.recv_from(&mut buf)?;

        // println!("{:?}", buf);
        // println!("amt {:?}", amt);
        // println!("src {:?}", src);

        print_sample_data_from_buf_both(&buf);

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

        timestamp_bytes =
          (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64).to_le_bytes();
        // TODO: there *MUST* be a better way of doing this!
        for x in 0..8 {
          outgoing_buf[x] = timestamp_bytes[x];
        }
        outgoing_buf[8] = outgoing_sequence_number.to_le_bytes()[0];
        outgoing_buf[9] = outgoing_sequence_number.to_le_bytes()[1];
        socket.send_to(&outgoing_buf, &src)?;
      }

    }
    Ok(())
}
