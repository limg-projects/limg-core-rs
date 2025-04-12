mod logic;

use crate::header::{ImageHeaderInternal, IMAGE_FLAG_ENDIAN_BIT, IMAGE_FLAG_USE_TRANSPARENT_BIT, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE};
use crate::spec::{DataEndian, ImageSpec};
use crate::pixel::{ColorType, PIXEL_BYTES};
use crate::error::{Error, Result};

use logic::{
    decode_to_rgb888_be,   decode_to_rgb888_le,
    decode_to_rgb565_be,   decode_to_rgb565_le,
    decode_to_rgba8888_be, decode_to_rgba8888_le,
    decode_to_rgba8888_alpha_be, decode_to_rgba8888_alpha_le,
};

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

    let header_ptr = data.as_ptr().cast::<ImageHeaderInternal>();
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

pub unsafe fn decode_data_unchecked(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) -> usize {
    let num_pixels = spec.num_pixels();
    let data = data.as_ptr();
    let buf = buf.as_mut_ptr();

    unsafe {
        match spec.data_endian {
            DataEndian::Big => {
                match color_type {
                    ColorType::Rgb888 => decode_to_rgb888_be(data, buf, num_pixels),
                    ColorType::Rgb565 => decode_to_rgb565_be(data, buf, num_pixels),
                    ColorType::Rgba8888 => {
                        if let Some(transparent_color) = spec.transparent_color {
                            decode_to_rgba8888_alpha_be(data, buf, transparent_color, num_pixels)
                        } else {
                            decode_to_rgba8888_be(data, buf, num_pixels)
                        }
                    }
                }
            },
            DataEndian::Little => {
                match color_type {
                    ColorType::Rgb888 => decode_to_rgb888_le(data, buf, num_pixels),
                    ColorType::Rgb565 => decode_to_rgb565_le(data, buf, num_pixels),
                    ColorType::Rgba8888 => {
                        if let Some(transparent_color) = spec.transparent_color {
                            decode_to_rgba8888_alpha_le(data, buf, transparent_color, num_pixels)
                        } else {
                            decode_to_rgba8888_le(data, buf, num_pixels)
                        }
                    }
                }
            },
        }
    }
    
    color_type.bytes_per_pixel() * num_pixels
}
