pub mod types;
pub mod wav;
pub mod decoder;
pub mod encoder;

pub use types::{WavHeader, Mp3Info};
pub use wav::{read_wav_header, write_wav};
pub use decoder::decode_mp3;
pub use encoder::encode_mp3;
