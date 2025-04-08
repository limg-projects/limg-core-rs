#![cfg_attr(not(test), no_std)]

mod header;
mod spec;
mod pixel;
mod encode;
mod decode;
mod error;

pub use crate::header::{IMAGE_SIGNATURE, IMAGE_HEADER_SIZE, IMAGE_CURRENT_VARSION, IMAGE_FLAG_ENDIAN_BIT, ImageHeader};
pub use crate::spec::{DataEndian, ImageSpec};
pub use crate::pixel::*;
pub use encode::*;
pub use decode::*;
pub use error::{Result, Error};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
