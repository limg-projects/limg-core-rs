mod scalar;

use crate::header::{ImageHeaderInternal, IMAGE_FLAG_ENDIAN_BIT, IMAGE_FLAG_USE_TRANSPARENT_BIT, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE};
use crate::spec::{DataEndian, ImageSpec};
use crate::pixel::{ColorType, PIXEL_BYTES};
use crate::error::{Error, Result};

use scalar::{
    decode_to_rgb888_be,   decode_to_rgb888_le,
    decode_to_rgb565_be,   decode_to_rgb565_le,
    decode_to_rgba8888_be, decode_to_rgba8888_le,
    decode_to_rgba8888_alpha_be, decode_to_rgba8888_alpha_le,
};

#[inline]
pub fn decode_to_buffer(data: impl AsRef<[u8]>, buf: &mut impl AsMut<[u8]>, color_type: ColorType) -> Result<(ImageSpec, usize)> {
    let data = data.as_ref();

    let header = from_data_header(data)?;
    let spec = header_to_spec(&header)?;
    let num_pixels = spec.num_pixels();

    let data = unsafe { data.get_unchecked(IMAGE_HEADER_SIZE..) };

    if data.len() < num_pixels * PIXEL_BYTES {
        return Err(Error::InputBufferTooSmall);
    }

    let buf = buf.as_mut();
    let written_size = num_pixels * color_type.bytes_per_pixel();

    if buf.len() < written_size {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe { decode_image(data, buf, &spec, color_type); }

    Ok((spec, written_size))
}

fn from_data_header(data: &[u8]) -> Result<ImageHeaderInternal> {
    if data.len() < IMAGE_HEADER_SIZE {
        return Err(Error::InputBufferTooSmall);
    }

    let header_ptr = data.as_ptr().cast::<ImageHeaderInternal>();
    let header = unsafe { header_ptr.read_unaligned() };

    Ok(header)
}

#[inline(always)]
fn header_to_spec(header: &ImageHeaderInternal) -> Result<ImageSpec> {
    // シグネチャが一致しない場合はエラー
    // 幅か高さが0の場合もエラー
    // 0の場合エンディアン関係なく0なのでそのまま比較
    if header.signature != IMAGE_SIGNATURE_U32_NE || header.width == 0 || header.height == 0 {
        return Err(Error::UnsupportedFormat);
    }

    let data_endian = unsafe { ::core::mem::transmute(header.flag & IMAGE_FLAG_ENDIAN_BIT) };
    let use_transparent = (header.flag & IMAGE_FLAG_USE_TRANSPARENT_BIT) != 0;
    let transparent_color = if use_transparent { Some(header.transparent_color) } else { None };

    Ok(ImageSpec {
        width: header.width.to_le(),
        height: header.height.to_le(),
        transparent_color,
        data_endian
    })
}

#[inline(always)]
unsafe fn decode_image(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) {
    let num_pixels = spec.num_pixels();

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
}
