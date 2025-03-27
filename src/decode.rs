
use crate::{Error, ImageHeaderInternal, ImageSpec, Result, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE };

pub fn decode_header(src: &[u8], decoded_size: Option<&mut usize>) -> Result<ImageSpec> {
    if src.len() < IMAGE_HEADER_SIZE {
        return Err(Error::InputBufferTooSmall);
    }

    let header = unsafe { decode_header_internal(src) };

    if header.signature != IMAGE_SIGNATURE_U32_NE || header.width == 0 || header.height == 0 {
        return Err(Error::UnsupportFormat);
    }

    if let Some(decoded_size) = decoded_size {
        *decoded_size = IMAGE_HEADER_SIZE;
    }

    Ok(ImageSpec {
        width: header.width.to_le(),
        height: header.height.to_le(),
        transparent_color: header.transparent_color.to_le()
    })
}

pub unsafe fn decode_header_unchecked(src: &[u8], decoded_size: Option<&mut usize>) -> ImageSpec {
    let header = unsafe { decode_header_internal(src) };

    if let Some(decoded_size) = decoded_size {
        *decoded_size = IMAGE_HEADER_SIZE;
    }

    ImageSpec {
        width: header.width.to_le(),
        height: header.height.to_le(),
        transparent_color: header.transparent_color.to_le()
    }
}

#[inline(always)]
unsafe fn decode_header_internal(src: &[u8]) -> ImageHeaderInternal {
    let mut header_uninit = ::core::mem::MaybeUninit::<ImageHeaderInternal>::uninit();
    unsafe {
        ::core::ptr::copy_nonoverlapping(src.as_ptr(), header_uninit.as_mut_ptr().cast::<u8>(), IMAGE_HEADER_SIZE);
        header_uninit.assume_init()
    }
}