#![cfg_attr(not(test), no_std)]

mod common;
pub mod pixel;
mod encode;
mod decode;
mod error;

pub use common::color::ColorType;
pub use common::header::{HEADER_SIZE, CURRENT_VARSION};
pub use common::spec::{ImageSpec, DataEndian};

pub use encode::{encode, encode_header, encode_data, encoded_size};
pub use decode::{decode, decode_header, decode_data, decoded_size};

pub use error::{Result, Error};