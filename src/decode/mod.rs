mod logic;

use crate::common::header::{ImageHeader, IMAGE_FLAG_ENDIAN_BIT, IMAGE_FLAG_USE_TRANSPARENT_BIT, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE};
use crate::spec::ImageSpec;
use crate::pixel::{ColorType, PIXEL_BYTES};
use crate::error::{Error, Result};

#[inline]
pub fn decode(data: &[u8], buf: &mut [u8], color_type: ColorType) -> Result<(ImageSpec, usize)> {
    let spec = decode_header(data)?;

    let data = unsafe { data.get_unchecked(IMAGE_HEADER_SIZE..) };
    let written_size = decode_data(data, buf, &spec, color_type)?;

    Ok((spec, written_size))
}

pub fn decode_header(data: &[u8]) -> Result<ImageSpec> {
    if data.len() < IMAGE_HEADER_SIZE {
        return Err(Error::InputBufferTooSmall);
    }

    let header_ptr = data.as_ptr().cast::<ImageHeader>();
    let header = unsafe { header_ptr.read_unaligned() };

    if header.signature != IMAGE_SIGNATURE_U32_NE {
        return Err(Error::UnsupportedFormat);
    }

    // エンディアン関係なく0はチェック可能
    if header.width == 0 || header.height == 0 {
        return Err(Error::UnsupportedFormat);
    }

    let transparent_color = if (header.flag & IMAGE_FLAG_USE_TRANSPARENT_BIT) != 0 { Some(header.transparent_color) } else { None };
    let data_endian = unsafe { ::core::mem::transmute(header.flag & IMAGE_FLAG_ENDIAN_BIT) };

    Ok(ImageSpec {
        width: u16::from_le(header.width),
        height: u16::from_le(header.height),
        transparent_color,
        data_endian
    })
}

#[inline]
pub fn decode_data(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) -> Result<usize> {
    let num_pixels = spec.num_pixels();

    if data.len() < PIXEL_BYTES * num_pixels {
        return Err(Error::InputBufferTooSmall);
    }

    if buf.len() < color_type.bytes_per_pixel() * num_pixels {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe { Ok(decode_data_unchecked(data, buf, spec, color_type)) }
}

#[inline(always)]
unsafe fn decode_data_unchecked(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) -> usize {
    unsafe { logic::decode_logic(data.as_ptr(), buf.as_mut_ptr(), spec, color_type) }
}
