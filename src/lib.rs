#![cfg_attr(not(test), no_std)]

mod common;
mod spec;
pub mod pixel;
pub mod encode;
pub mod decode;
mod error;

pub use crate::spec::{DataEndian, ImageSpec};
pub use error::{Result, Error};
