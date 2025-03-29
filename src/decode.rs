use crate::header::{ImageHeaderInternal, IMAGE_FLAG_ENDIAN_BIT, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE};
use crate::spec::{DataEndian, ImageSpec};
use crate::pixel::{RGB_CHANNELS, PIXEL_BYTES, pixel_to_rgb};
use crate::error::{Error, Result};
use ::core::slice::from_raw_parts;

pub fn decode(image_data: &[u8], rgb_buf: &mut [u8], consumed_bytes: Option<&mut usize>) -> Result<(ImageSpec, usize)> {
    let mut total_consumed_bytes = 0;
    let mut data_consumed_bytes = 0;

    let spec = decode_header(image_data, Some(&mut data_consumed_bytes))?;
    total_consumed_bytes += data_consumed_bytes;

    let pixel_data = match image_data.get(IMAGE_HEADER_SIZE..) {
        Some(pixel_data) => pixel_data,
        None => return Err(Error::InputBufferTooSmall),
    };
    let written_size = decode_data_raw(pixel_data, rgb_buf, spec.num_pixels(), spec.data_endian, Some(&mut data_consumed_bytes))?;
    total_consumed_bytes += data_consumed_bytes;

    if let Some(consumed_bytes) = consumed_bytes {
        *consumed_bytes = total_consumed_bytes;
    }

    Ok((spec, written_size))
}

pub unsafe fn decode_unchecked(image_data: &[u8], rgb_buf: &mut [u8], consumed_bytes: Option<&mut usize>) -> (ImageSpec, usize) {
    let mut total_consumed_bytes = 0;
    let mut data_consumed_bytes = 0;

    let spec = unsafe { decode_header_unchecked(image_data, Some(&mut data_consumed_bytes)) };
    total_consumed_bytes += data_consumed_bytes;
    let mut written_size = IMAGE_HEADER_SIZE;

    let pixel_data = unsafe { image_data.get_unchecked(IMAGE_HEADER_SIZE..) };
    written_size += unsafe { decode_data_raw_unchecked(pixel_data, rgb_buf, spec.num_pixels(), spec.data_endian, Some(&mut data_consumed_bytes)) };
    total_consumed_bytes += data_consumed_bytes;

    if let Some(consumed_bytes) = consumed_bytes {
        *consumed_bytes = total_consumed_bytes;
    }

    (spec, written_size)
}

pub fn decode_header(data: &[u8], consumed_bytes: Option<&mut usize>) -> Result<ImageSpec> {
    if data.len() < IMAGE_HEADER_SIZE {
        return Err(Error::InputBufferTooSmall);
    }

    let header = unsafe { decode_header_internal(data) };

    if header.signature != IMAGE_SIGNATURE_U32_NE || header.width == 0 || header.height == 0 {
        return Err(Error::UnsupportFormat);
    }

    if let Some(consumed_bytes) = consumed_bytes {
        *consumed_bytes = IMAGE_HEADER_SIZE;
    }

    Ok(ImageSpec {
        width: header.width.to_le(),
        height: header.height.to_le(),
        transparent_color: header.transparent_color.to_le(),
        data_endian: unsafe { ::core::mem::transmute(header.flag & IMAGE_FLAG_ENDIAN_BIT) },
    })
}

pub unsafe fn decode_header_unchecked(data: &[u8], consumed_bytes: Option<&mut usize>) -> ImageSpec {
    let header = unsafe { decode_header_internal(data) };

    if let Some(consumed_bytes) = consumed_bytes {
        *consumed_bytes = IMAGE_HEADER_SIZE;
    }

    ImageSpec {
        width: header.width.to_le(),
        height: header.height.to_le(),
        transparent_color: header.transparent_color.to_le(),
        data_endian: unsafe { ::core::mem::transmute(header.flag & IMAGE_FLAG_ENDIAN_BIT) },
    }
}

#[inline(always)]
unsafe fn decode_header_internal(data: &[u8]) -> ImageHeaderInternal {
    let mut header = ::core::mem::MaybeUninit::<ImageHeaderInternal>::uninit();
    unsafe {
        ::core::ptr::copy_nonoverlapping(data.as_ptr(), header.as_mut_ptr().cast::<u8>(), IMAGE_HEADER_SIZE);
        header.assume_init()
    }
}

macro_rules! decode_data_endian {
    ($data: ident, $data_unchecked: ident, $data_raw: ident, $data_raw_unchecked: ident, $from_byte_fn: ident) => {
        #[inline]
        pub fn $data(pixel_data: &[u16], rgb_buf: &mut [u8], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> Result<usize> {
            let pixel_data = unsafe {
                from_raw_parts(pixel_data.as_ptr().cast::<u8>(), pixel_data.len() * PIXEL_BYTES)
            };
            $data_raw(pixel_data, rgb_buf, num_pixels, consumed_bytes)
        }

        #[inline]
        pub unsafe fn $data_unchecked(pixel_data: &[u16], rgb_buf: &mut [u8], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> usize {
            let pixel_data = unsafe { from_raw_parts(pixel_data.as_ptr().cast::<u8>(), pixel_data.len() * PIXEL_BYTES) };
            unsafe { $data_raw_unchecked(pixel_data, rgb_buf, num_pixels, consumed_bytes) }
        }

        #[inline]
        pub fn $data_raw(pixel_data: &[u8], rgb_buf: &mut [u8], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> Result<usize> {
            if pixel_data.len() < num_pixels * PIXEL_BYTES {
                return Err(Error::InputBufferTooSmall);
            }
            if rgb_buf.len() < num_pixels * RGB_CHANNELS {
                return Err(Error::OutputBufferTooSmall);
            }
        
            unsafe {
                Ok($data_raw_unchecked(pixel_data, rgb_buf, num_pixels, consumed_bytes))
            }
        }

        pub unsafe fn $data_raw_unchecked(pixel_data: &[u8], rgb_buf: &mut [u8], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> usize {
            let mut pixels_ptr = pixel_data.as_ptr().cast::<[u8; PIXEL_BYTES]>();
            let mut rgbs_ptr = rgb_buf.as_mut_ptr().cast::<[u8; RGB_CHANNELS]>();
        
            for _ in 0..num_pixels {
                unsafe {
                    *rgbs_ptr = pixel_to_rgb(u16::$from_byte_fn(*pixels_ptr));
                    pixels_ptr = pixels_ptr.add(1);
                    rgbs_ptr = rgbs_ptr.add(1);
                }
            }

            if let Some(consumed_bytes) = consumed_bytes {
                *consumed_bytes = num_pixels * PIXEL_BYTES;
            }

            num_pixels * RGB_CHANNELS
        }
    };
}

#[inline]
pub fn decode_data(pixel_data: &[u16], rgb_buf: &mut [u8], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    let pixel_data = unsafe {
        from_raw_parts(pixel_data.as_ptr().cast::<u8>(), pixel_data.len() * PIXEL_BYTES)
    };
    decode_data_raw(pixel_data, rgb_buf, num_pixels, data_endian, consumed_bytes)
}

#[inline]
pub unsafe fn decode_data_unchecked(pixel_data: &[u16], rgb_buf: &mut [u8], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> usize {
    match data_endian {
        DataEndian::Big => unsafe { decode_data_be_unchecked(pixel_data, rgb_buf, num_pixels, consumed_bytes) },
        DataEndian::Little => unsafe { decode_data_le_unchecked(pixel_data, rgb_buf, num_pixels, consumed_bytes) },
    }
}

#[inline]
pub fn decode_data_raw(pixel_data: &[u8], rgb_buf: &mut [u8], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if pixel_data.len() < num_pixels * PIXEL_BYTES {
        return Err(Error::InputBufferTooSmall);
    }
    if rgb_buf.len() < num_pixels * RGB_CHANNELS {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(decode_data_raw_unchecked(pixel_data, rgb_buf, num_pixels, data_endian, consumed_bytes))
    }
}

#[inline]
pub unsafe fn decode_data_raw_unchecked(pixel_data: &[u8], rgb_buf: &mut [u8], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> usize {
    match data_endian {
        DataEndian::Big => unsafe { decode_data_raw_be_unchecked(pixel_data, rgb_buf, num_pixels, consumed_bytes) },
        DataEndian::Little => unsafe { decode_data_raw_le_unchecked(pixel_data, rgb_buf, num_pixels, consumed_bytes) },
    }
}

decode_data_endian! {
    decode_data_be,
    decode_data_be_unchecked,
    decode_data_raw_be,
    decode_data_raw_be_unchecked,
    from_be_bytes
}

decode_data_endian! {
    decode_data_le,
    decode_data_le_unchecked,
    decode_data_raw_le,
    decode_data_raw_le_unchecked,
    from_le_bytes
}