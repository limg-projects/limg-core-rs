#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

mod common;
mod encodes;
mod decodes;
mod error;

pub use common::color::ColorType;
pub use common::header::{HEADER_SIZE, CURRENT_VARSION};
pub use common::spec::{ImageSpec, PixelEndian};
pub use common::pixel::{pixel_to_rgb, rgb_to_pixel, PIXEL_BYTES};

pub use encodes::{encode, encode_header, encode_data, encoded_size};
pub use decodes::{decode, decode_header, decode_data, decoded_size};

pub use error::{Result, Error};