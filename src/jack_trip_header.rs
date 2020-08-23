use std::fmt;
use crate::sampling_rate_t::*;

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
