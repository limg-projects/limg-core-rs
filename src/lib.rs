#![cfg_attr(not(test), no_std)]

//! Limg 画像を読み書きするための`no_std`コアライブラリです。
//! 
//! 読み書きは`RGB888`、`RGB565`、`RGBA8888`に対応しています。

mod common;
mod encode;
mod decode;
mod error;

pub use common::color::ColorType;
pub use common::header::{HEADER_SIZE, CURRENT_VARSION};
pub use common::spec::{ImageSpec, PixelEndian};
pub use common::pixel::{pixel_to_rgb, rgb_to_pixel, PIXEL_BYTES};

pub use encode::{encode, encode_header, encode_data, encoded_size};
pub use decode::{decode, decode_header, decode_data, decoded_size};

pub use error::{Result, Error};