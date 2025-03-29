#![no_std]

mod header;
mod spec;
mod pixel;
mod encode;
mod decode;
mod error;

pub use crate::header::{IMAGE_SIGNATURE, IMAGE_HEADER_SIZE, ImageHeader};
pub use crate::spec::ImageSpec;
pub use encode::*;
pub use decode::*;
pub use error::*;

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
