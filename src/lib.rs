#![cfg_attr(not(test), no_std)]

mod common;
pub mod spec;
pub mod pixel;
pub mod encode;
pub mod decode;
mod error;

pub use error::{Result, Error};
