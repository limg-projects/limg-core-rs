use crate::{Error, ImageSpec, Result};
use crate::header::{IMAGE_SIGNATURE_U32_NE, IMAGE_HEADER_SIZE, ImageHeaderInternal};
use crate::pixel::{RGB_CHANNELS, PIXEL_BYTES, rgb_to_pixel};
use ::core::slice::from_raw_parts_mut;
use ::core::ptr::copy_nonoverlapping;

#[inline(always)]
pub const fn encode_bounds(spec: &ImageSpec) -> usize {
    IMAGE_HEADER_SIZE + spec.num_pixels() * PIXEL_BYTES
}

#[inline]
pub fn encode(rgb_data: &[u8], image_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }
    if rgb_data.len() < spec.num_pixels() * RGB_CHANNELS {
        return Err(Error::InputBufferTooSmall);
    }
    if image_buf.len() < encode_bounds(spec) {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_unchecked(rgb_data, image_buf, spec, consumed_bytes))
    }
}

#[inline]
pub unsafe fn encode_unchecked(rgb_data: &[u8], image_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> usize {
    let mut written_bytes = 0;
    unsafe {
        written_bytes += encode_header_unchecked(image_buf, spec);

        let data_slice = image_buf.get_unchecked_mut(IMAGE_HEADER_SIZE..);
        written_bytes += encode_data_raw_unchecked(rgb_data, data_slice, spec.num_pixels(), consumed_bytes);
    }
    written_bytes
}

#[inline]
pub fn encode_header(buf: &mut [u8], spec: &ImageSpec) -> Result<usize> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }
    if buf.len() < IMAGE_HEADER_SIZE {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_header_unchecked(buf, spec))
    }
}

pub unsafe fn encode_header_unchecked(buf: &mut [u8], spec: &ImageSpec) -> usize {
    let header = ImageHeaderInternal {
        signature: IMAGE_SIGNATURE_U32_NE,
        width: spec.width.to_le(),
        height: spec.height.to_le(),
        transparent_color: spec.transparent_color.to_le(),
        reserve: 0
    };

    let header_ptr = (&header as *const ImageHeaderInternal).cast::<u8>();

    unsafe {
        copy_nonoverlapping(header_ptr, buf.as_mut_ptr(), IMAGE_HEADER_SIZE);
    }

    IMAGE_HEADER_SIZE
}

#[inline]
pub fn encode_data(rgb_data: &[u8], pixel_buf: &mut [u16], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if rgb_data.len() < num_pixels * RGB_CHANNELS {
        return Err(Error::InputBufferTooSmall);
    }
    if pixel_buf.len() < num_pixels {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_data_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes))
    }
}

#[inline]
pub unsafe fn encode_data_unchecked(rgb_data: &[u8], pixel_buf: &mut [u16], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> usize {
    unsafe {
        let pixel_buf = from_raw_parts_mut(pixel_buf.as_mut_ptr().cast::<u8>(), pixel_buf.len() * PIXEL_BYTES);
        encode_data_raw_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes)
    }
}

#[inline]
pub fn encode_data_raw(rgb_data: &[u8], pixel_buf: &mut [u8], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if rgb_data.len() < num_pixels * RGB_CHANNELS {
        return Err(Error::InputBufferTooSmall);
    }
    if pixel_buf.len() < num_pixels * PIXEL_BYTES {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_data_raw_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes))
    }
}

pub unsafe fn encode_data_raw_unchecked(rgb_data: &[u8], pixel_buf: &mut [u8], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> usize {
    let mut rgb_ptr = rgb_data.as_ptr().cast::<[u8; RGB_CHANNELS]>();
    let mut pixel_ptr = pixel_buf.as_mut_ptr().cast::<[u8; PIXEL_BYTES]>();

    for _ in 0..num_pixels {
        unsafe {
            *pixel_ptr = rgb_to_pixel(*rgb_ptr).to_le_bytes();
            rgb_ptr = rgb_ptr.add(1);
            pixel_ptr = pixel_ptr.add(1);
        }
    }

    if let Some(consumed_bytes) = consumed_bytes {
        *consumed_bytes = num_pixels * RGB_CHANNELS;
    }

    num_pixels * PIXEL_BYTES
}