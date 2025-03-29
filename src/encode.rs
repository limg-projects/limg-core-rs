use crate::header::{ImageHeaderInternal, IMAGE_CURRENT_VARSION, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE};
use crate::spec::{DataEndian, ImageSpec};
use crate::pixel::{RGB_CHANNELS, PIXEL_BYTES, rgb_to_pixel};
use crate::error::{Error, Result};
use ::core::slice::from_raw_parts_mut;

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
        written_bytes += encode_data_raw_unchecked(rgb_data, data_slice, spec.num_pixels(), spec.data_endian, consumed_bytes);
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
        version: IMAGE_CURRENT_VARSION,
        flag: spec.flag(),
        width: spec.width.to_le(),
        height: spec.height.to_le(),
        transparent_color: spec.transparent_color.to_le(),
    };

    let header_ptr = (&header as *const ImageHeaderInternal).cast::<u8>();

    unsafe {
        ::core::ptr::copy_nonoverlapping(header_ptr, buf.as_mut_ptr(), IMAGE_HEADER_SIZE);
    }

    IMAGE_HEADER_SIZE
}


macro_rules! encode_data_endian {
    ($data: ident, $data_unchecked: ident, $data_raw: ident, $data_raw_unchecked: ident, $to_byte_fn: ident) => {
        #[inline]
        pub fn $data(rgb_data: &[u8], pixel_buf: &mut [u16], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> Result<usize> {
            let pixel_buf = unsafe {
                from_raw_parts_mut(pixel_buf.as_mut_ptr().cast::<u8>(), pixel_buf.len() * PIXEL_BYTES)
            };
            $data_raw(rgb_data, pixel_buf, num_pixels, consumed_bytes)
        }
        
        #[inline]
        pub unsafe fn $data_unchecked(rgb_data: &[u8], pixel_buf: &mut [u16], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> usize {
            let pixel_buf = unsafe {
                from_raw_parts_mut(pixel_buf.as_mut_ptr().cast::<u8>(), pixel_buf.len() * PIXEL_BYTES)
            };
            unsafe { $data_raw_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes) }
        }

        #[inline]
        pub fn $data_raw(rgb_data: &[u8], pixel_buf: &mut [u8], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> Result<usize> {
            if rgb_data.len() < num_pixels * RGB_CHANNELS {
                return Err(Error::InputBufferTooSmall);
            }
            if pixel_buf.len() < num_pixels * PIXEL_BYTES {
                return Err(Error::OutputBufferTooSmall);
            }
        
            unsafe {
                Ok($data_raw_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes))
            }
        }

        pub unsafe fn $data_raw_unchecked(rgb_data: &[u8], pixel_buf: &mut [u8], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> usize {
            let mut rgb_ptr = rgb_data.as_ptr().cast::<[u8; RGB_CHANNELS]>();
            let mut pixel_ptr = pixel_buf.as_mut_ptr().cast::<[u8; PIXEL_BYTES]>();
        
            for _ in 0..num_pixels {
                unsafe {
                    *pixel_ptr = rgb_to_pixel(*rgb_ptr).$to_byte_fn();
                    rgb_ptr = rgb_ptr.add(1);
                    pixel_ptr = pixel_ptr.add(1);
                }
            }
        
            if let Some(consumed_bytes) = consumed_bytes {
                *consumed_bytes = num_pixels * RGB_CHANNELS;
            }
        
            num_pixels * PIXEL_BYTES
        }
    };
}

#[inline]
pub fn encode_data(rgb_data: &[u8], pixel_buf: &mut [u16], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    let pixel_buf = unsafe {
        from_raw_parts_mut(pixel_buf.as_mut_ptr().cast::<u8>(), pixel_buf.len() * PIXEL_BYTES)
    };
    encode_data_raw(rgb_data, pixel_buf, num_pixels, data_endian, consumed_bytes)
}

#[inline]
pub unsafe fn encode_data_unchecked(rgb_data: &[u8], pixel_buf: &mut [u16], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> usize {
    let pixel_buf = unsafe {
        from_raw_parts_mut(pixel_buf.as_mut_ptr().cast::<u8>(), pixel_buf.len() * PIXEL_BYTES)
    };
    unsafe { encode_data_raw_unchecked(rgb_data, pixel_buf, num_pixels, data_endian, consumed_bytes) }
}

#[inline]
pub fn encode_data_raw(rgb_data: &[u8], pixel_buf: &mut [u8], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if rgb_data.len() < num_pixels * RGB_CHANNELS {
        return Err(Error::InputBufferTooSmall);
    }
    if pixel_buf.len() < num_pixels * PIXEL_BYTES {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_data_raw_unchecked(rgb_data, pixel_buf, num_pixels, data_endian, consumed_bytes))
    }
}

#[inline]
pub unsafe fn encode_data_raw_unchecked(rgb_data: &[u8], pixel_buf: &mut [u8], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> usize {
    match data_endian {
        DataEndian::Big => unsafe { encode_data_raw_be_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes) },
        DataEndian::Little => unsafe { encode_data_raw_le_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes) },
    }
}

encode_data_endian! {
    encode_data_be,
    encode_data_be_unchecked,
    encode_data_raw_be,
    encode_data_raw_be_unchecked,
    to_be_bytes
}

encode_data_endian! {
    encode_data_le,
    encode_data_le_unchecked,
    encode_data_raw_le,
    encode_data_raw_le_unchecked,
    to_le_bytes
}