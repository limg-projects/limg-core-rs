mod logic;

use crate::common::header::{ImageHeader, IMAGE_CURRENT_VARSION, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE};
use crate::spec::{DataEndian, ImageSpec};
use crate::pixel::{ColorType, PIXEL_BYTES};
use crate::error::{Error, Result};

use logic::{
    encode_from_rgb888_be,   encode_from_rgb888_le,
    encode_from_rgb565_be,   encode_from_rgb565_le,
    encode_from_rgba8888_be, encode_from_rgba8888_le,
};

/// Calculates the total number of bytes needed to encode an image with the given specification.
///
/// This includes both the image header and the pixel data region.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{ImageSpec, encoded_size, rgb_to_pixel};
/// 
/// let spec = ImageSpec::new(100, 100, rgb_to_pixel([0, 0, 0]));
/// let bounds = encoded_size(&spec);
/// 
/// // HeaderSize(12) + width(100) * height(100) * PixelSize(2)
/// assert_eq!(bounds, 20012);
/// ```
#[inline(always)]
pub const fn encoded_size(spec: &ImageSpec) -> usize {
    IMAGE_HEADER_SIZE + spec.num_pixels() * PIXEL_BYTES
}

#[inline]
pub fn encode(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) -> Result<usize> {
    let num_pixels = spec.width as usize * spec.height as usize;

    if num_pixels == 0 {
        return Err(Error::ZeroImageDimensions);
    }
    if data.len() < color_type.bytes_per_pixel() * num_pixels {
        return Err(Error::InputBufferTooSmall);
    }
    if buf.len() < IMAGE_HEADER_SIZE + PIXEL_BYTES * num_pixels {
        return Err(Error::OutputBufferTooSmall);
    }

    let mut written_size = 0;

    unsafe {
        written_size += encode_header_unchecked(buf.get_unchecked_mut(..IMAGE_HEADER_SIZE), spec);
        written_size += encode_data_unchecked(data, buf.get_unchecked_mut(IMAGE_HEADER_SIZE..), num_pixels, spec.data_endian, color_type);
    }

    debug_assert_eq!(written_size, encoded_size(spec));

    Ok(written_size)
}

#[inline]
pub fn encode_header(buf: &mut [u8], spec: &ImageSpec) -> Result<usize> {
    let num_pixel = spec.width as usize * spec.height as usize;

    if num_pixel == 0 {
        return Err(Error::ZeroImageDimensions);
    }

    if buf.len() < IMAGE_HEADER_SIZE {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_header_unchecked(buf, spec))
    }
}

unsafe fn encode_header_unchecked(buf: &mut [u8], spec: &ImageSpec) -> usize {
    let header = ImageHeader {
        signature: IMAGE_SIGNATURE_U32_NE,
        version: IMAGE_CURRENT_VARSION,
        flag: spec.flag(),
        width: spec.width.to_le(),
        height: spec.height.to_le(),
        transparent_color: spec.transparent_color.unwrap_or(0).to_le(),
    };

    let header_ptr = buf.as_mut_ptr().cast::<ImageHeader>();

    unsafe { header_ptr.write_unaligned(header); }

    IMAGE_HEADER_SIZE
}

#[inline]
pub fn encode_data(data: &[u8], buf: &mut [u8], num_pixels: usize, data_endian: DataEndian, color_type: ColorType) -> Result<usize> {
    if data.len() < color_type.bytes_per_pixel() * num_pixels {
        return Err(Error::InputBufferTooSmall);
    }

    if buf.len() < PIXEL_BYTES * num_pixels {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_data_unchecked(data, buf, num_pixels, data_endian, color_type))
    }
}

pub unsafe fn encode_data_unchecked(data: &[u8], buf: &mut [u8], num_pixels: usize, data_endian: DataEndian, color_type: ColorType) -> usize {
    let data = data.as_ptr();
    let buf = buf.as_mut_ptr();

    unsafe {
        match data_endian {
            DataEndian::Big => {
                match color_type {
                    ColorType::Rgb888 => encode_from_rgb888_be(data, buf, num_pixels),
                    ColorType::Rgb565 => encode_from_rgb565_be(data, buf, num_pixels),
                    ColorType::Rgba8888 => encode_from_rgba8888_be(data, buf, num_pixels),
                }
            },
            DataEndian::Little => {
                match color_type {
                    ColorType::Rgb888 => encode_from_rgb888_le(data, buf, num_pixels),
                    ColorType::Rgb565 => encode_from_rgb565_le(data, buf, num_pixels),
                    ColorType::Rgba8888 => encode_from_rgba8888_le(data, buf, num_pixels),
                }
            },
        }
    }

    PIXEL_BYTES * color_type.bytes_per_pixel()
}
