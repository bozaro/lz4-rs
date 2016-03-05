extern crate libc;

pub mod liblz4;
mod decoder;
mod encoder;

pub use decoder::Decoder;
pub use encoder::Encoder;
pub use encoder::EncoderBuilder;
pub use liblz4::BlockSize;
pub use liblz4::BlockMode;
pub use liblz4::ContentChecksum;
pub use liblz4::version;
