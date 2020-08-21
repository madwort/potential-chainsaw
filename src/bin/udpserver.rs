use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::fmt;

// Sampling Rates supported by JACK
// Copied from jacktrip/AudioInterface.h
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
  // assume bit res 16 (u16 elements) & max buffer size 256 (array size 256)
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

fn main() -> std::io::Result<()> {
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
        let mut count = 0;

        // for x in s.data.iter() {
        //  println!("{} - {:?}", count, x);
        // }

        println!("Buffer size u8 {}", s.buffer_size);

        for x in 0..s.buffer_size as usize {
          println!("{} - {:?} - {:?}", x, s.data[x], s.jack_data(x));
          count = count+1;
        }
        // println!("{:?}", buf);
        // println!("Struct: {:?}", s);

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
        // break;
      }

    } // the socket is closed here
    Ok(())
}
