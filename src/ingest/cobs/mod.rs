//! COBS streaming and encoding module.

pub mod codec;
pub mod reader;

pub use codec::{cobs_decode, cobs_encode};
pub use reader::SerialCobsReader;
