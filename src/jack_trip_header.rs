use std::fmt;

// Sampling Rates supported by JACK
// Copied from jacktrip/AudioInterface.h
pub enum SamplingRateT {
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
      SamplingRateT::UNDEF => write!(f, "UNKNOWN SAMPLE RATE OMG!")
    }
  }
}

#[repr(C, packed)]
// #[derive(Debug)]
pub struct JackTripHeader {
  pub time_stamp: u64, ///< Time Stamp
  pub sequence_number: u16, ///< Sequence Number
  pub buffer_size: u16, ///< Buffer Size in Samples
  pub sampling_rate: SamplingRateT, ///< Sampling Rate in JackAudioInterface::samplingRateT
  pub bit_resolution: u8, ///< Audio Bit Resolution
  pub num_channels: u8, ///< Number of Channels, we assume input and outputs are the same
  pub connection_mode: u8,
  // assume bit res 16 (u16 elements) & max buffer size 256 (array size 256)
  pub data: [i16; 256], // Jack frames per period size (typically 64/128/256 etc)
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
