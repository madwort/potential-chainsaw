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

impl SamplingRateT {
  pub fn as_numeric(&self) -> usize {
    let numeric_sample_rate = match self {
      SamplingRateT::SR22 => 22050,
      SamplingRateT::SR32 => 32000,
      SamplingRateT::SR44 => 44100,
      SamplingRateT::SR48 => 48000,
      SamplingRateT::SR88 => 88000,
      SamplingRateT::SR96 => 96000,
      SamplingRateT::SR192 => 192000,
      SamplingRateT::UNDEF => 0
    };
    return numeric_sample_rate;
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
pub struct JackTripHeader {
  pub time_stamp: u64, ///< Time Stamp
  pub sequence_number: u16, ///< Sequence Number
  pub buffer_size: u16, ///< Buffer Size in Samples
  pub sampling_rate: SamplingRateT, ///< Sampling Rate in JackAudioInterface::samplingRateT
  // only support bit_resolution = 16
  pub bit_resolution: u8, ///< Audio Bit Resolution
  // only support num_channels = 1 at the moment
  pub num_channels: u8, ///< Number of Channels, we assume input and outputs are the same
  pub connection_mode: u8,
  // assume bit res 16 (u16 elements) & max buffer size 256 (array size 256)
  pub data: [i16; 256], // Jack frames per period size (typically 64/128/256 etc)
}

impl JackTripHeader {
  pub fn get_jack_data(&self, index: usize) -> f32{
    if self.bit_resolution != 16 {
      panic!("We only support jacktrip packets with 16bit audio data!!");
    }
    self.data[index] as f32 / 32768.0
  }

  // TO REMOVE: this does not work! I do not understand memory management!
  // pub fn set_jack_data(&self, index: usize, datum: f32){
  //   if self.bit_resolution != 16 {
  //     panic!("We only support jacktrip packets with 16bit audio data!!");
  //   }
  //   self.data[index] = (datum * 32768.0) as i16;
  // }
}

impl fmt::Display for JackTripHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unsafe{
      writeln!(f, "{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {},{},{}",
        "time_stamp", self.time_stamp,
        "sequence_number", self.sequence_number,
        "buffer_size", self.buffer_size,
        "sampling_rate", self.sampling_rate,
        "bit_resolution", self.bit_resolution,
        "num_channels", self.num_channels,
        "connection_mode", self.connection_mode,
        "data (extract)", self.get_jack_data(0),self.get_jack_data(1),self.get_jack_data(2),
      )
    }
  }
}

impl From<[u8; 528]> for JackTripHeader {
  // TODO: check whether this is duplicating the array in memory
  fn from(item: [u8; 528]) -> Self {
    let s: JackTripHeader = unsafe { std::ptr::read(item.as_ptr() as *const _)};
    s
  }
}
