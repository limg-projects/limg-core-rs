mod scalar;

use crate::header::{ImageHeaderInternal, IMAGE_FLAG_ENDIAN_BIT, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE};
use crate::spec::{DataEndian, ImageSpec};
use crate::pixel::{RGB_CHANNELS, PIXEL_BYTES, pixel_to_rgb};
use crate::error::{Error, Result};
use crate::ColorType;
use ::core::slice::from_raw_parts;
use ::core::slice::from_raw_parts_mut;
use ::core::mem::MaybeUninit;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

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

#[cfg(feature = "alloc")]
#[inline]
pub fn decode_to_vec(data: impl AsRef<[u8]>, color_type: ColorType) -> Result<(ImageSpec, Vec<u8>)> {
    let data = data.as_ref();

    let header = from_data_header(data)?;
    let spec = header_to_spec(&header)?;
    let num_pixels = spec.num_pixels();

    let data = unsafe { data.get_unchecked(IMAGE_HEADER_SIZE..) };

    if data.len() < num_pixels * PIXEL_BYTES {
        return Err(Error::InputBufferTooSmall);
    }

    let vec = unsafe {
        let size = num_pixels * color_type.bytes_per_pixel();

        let mut buf = Vec::with_capacity(size);
        buf.set_len(size);

        decode_image(data, &mut buf, &spec, color_type);

        buf
    };

    Ok((spec, vec))
}

#[cfg(feature = "std")]
#[inline]
pub fn decode_from_read_to_buffer(read: &mut impl std::io::Read, buf: &mut impl AsMut<[u8]>, color_type: ColorType) -> Result<(ImageSpec, usize)> {
    let header = read_header(read)?;
    let spec = header_to_spec(&header)?;
    let num_pixels = spec.num_pixels();

    let buf = buf.as_mut();
    let written_size = num_pixels * color_type.bytes_per_pixel();

    if buf.len() < written_size {
        return Err(Error::OutputBufferTooSmall);
    }

    let mut data = Vec::with_capacity(num_pixels * PIXEL_BYTES);
    read.read_exact(&mut data)?;

    unsafe { decode_image(&data, buf, &spec, color_type); }

    Ok((spec, written_size))
}

#[cfg(feature = "std")]
#[inline]
pub fn decode_from_read_to_vec(read: &mut impl std::io::Read, color_type: ColorType) -> Result<(ImageSpec, Vec<u8>)> {
    let header = read_header(read)?;
    let spec = header_to_spec(&header)?;
    let num_pixels = spec.num_pixels();

    let mut data = Vec::with_capacity(num_pixels * PIXEL_BYTES);
    read.read_exact(&mut data)?;

    let vec = unsafe {
        let size = num_pixels * color_type.bytes_per_pixel();

        let mut buf = Vec::with_capacity(size);
        buf.set_len(size);

        decode_image(&data, &mut buf, &spec, color_type);

        buf
    };

    Ok((spec, vec))
}

#[cfg(feature = "std")]
#[inline]
pub fn decode_from_file_to_buffer(path: impl AsRef<std::path::Path>, buf: &mut impl AsMut<[u8]>, color_type: ColorType) -> Result<(ImageSpec, usize)> {
    let mut file = std::fs::File::open(path)?;
    decode_from_read_to_buffer(&mut file, buf, color_type)
}

#[cfg(feature = "std")]
#[inline]
pub fn decode_from_file_to_vec(path: impl AsRef<std::path::Path>, color_type: ColorType) -> Result<(ImageSpec, Vec<u8>)> {
    let mut file = std::fs::File::open(path)?;
    decode_from_read_to_vec(&mut file, color_type)
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
fn read_header(read: &mut impl std::io::Read) -> Result<ImageHeaderInternal> {
    let mut header = MaybeUninit::<ImageHeaderInternal>::uninit();
    let header_buf = unsafe { from_raw_parts_mut(header.as_mut_ptr().cast::<u8>(), IMAGE_HEADER_SIZE) };
    read.read_exact(header_buf)?;

    unsafe { Ok(header.assume_init()) }
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

    Ok(ImageSpec {
        width: header.width.to_le(),
        height: header.height.to_le(),
        transparent_color: header.transparent_color,
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
                    ColorType::Rgba8888 => decode_to_rgba8888_be(data, buf, num_pixels),
                }
            },
            DataEndian::Little => {
                match color_type {
                    ColorType::Rgb888 => decode_to_rgb888_le(data, buf, num_pixels),
                    ColorType::Rgb565 => decode_to_rgb565_le(data, buf, num_pixels),
                    ColorType::Rgba8888 => decode_to_rgba8888_le(data, buf, num_pixels),
                }
            },
        }
    }
}
