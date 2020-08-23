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
