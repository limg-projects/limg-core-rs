
use crate::{pixel_to_rgb, Error, ImageHeaderInternal, ImageSpec, Result, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE, PIXEL_SIZE, RGB_CHANNELS };

pub fn decode(image_data: &[u8], rgb_buf: &mut [u8], consumed_bytes: Option<&mut usize>) -> Result<(ImageSpec, usize)> {
    let mut result_size = 0;
    let mut result_consumed_size = 0;

    let spec = decode_header(image_data, Some(&mut result_consumed_size))?;
    result_size += result_consumed_size;
    let written_size = decode_data_raw(image_data, rgb_buf, &spec, Some(&mut result_consumed_size))?;

    if let Some(consumed_bytes) = consumed_bytes {
        *consumed_bytes = result_size;
    }

    Ok((spec, written_size))
}

pub unsafe fn decode_unchecked(image_data: &[u8], rgb_buf: &mut [u8], consumed_bytes: Option<&mut usize>) -> (ImageSpec, usize) {
    let mut consumed_bytes = consumed_bytes;

    unsafe {
        let spec = decode_header_unchecked(image_data, consumed_bytes.as_mut().map(|s| &mut **s));
        let pixel_data = image_data.get_unchecked(IMAGE_HEADER_SIZE..);
        let size = IMAGE_HEADER_SIZE + decode_data_raw_unchecked(pixel_data, rgb_buf, &spec, consumed_bytes.as_mut().map(|s| &mut **s));
        (spec, size)
    }
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
        transparent_color: header.transparent_color.to_le()
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
        transparent_color: header.transparent_color.to_le()
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

#[inline]
pub fn decode_data(pixel_data: &[u16], rgb_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    let pixel_data = unsafe {
        ::core::slice::from_raw_parts(pixel_data.as_ptr().cast::<u8>(), pixel_data.len() * PIXEL_SIZE)
    };
    decode_data_raw(pixel_data, rgb_buf, spec, consumed_bytes)
}

#[inline]
pub unsafe fn decode_data_unchecked(pixel_data: &[u16], rgb_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> usize {
    unsafe {
        let pixel_data = ::core::slice::from_raw_parts(pixel_data.as_ptr().cast::<u8>(), pixel_data.len() * size_of::<u16>());
        decode_data_raw_unchecked(pixel_data, rgb_buf, spec, consumed_bytes)
    }
}

#[inline]
pub fn decode_data_raw(pixel_data: &[u8], rgb_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }

    let num_pixels = spec.num_pixels();

    if pixel_data.len() < num_pixels * PIXEL_SIZE {
        return Err(Error::InputBufferTooSmall);
    }
    if rgb_buf.len() < num_pixels * RGB_CHANNELS {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(decode_data_raw_unchecked(pixel_data, rgb_buf, spec, consumed_bytes))
    }
}

pub unsafe fn decode_data_raw_unchecked(pixel_data: &[u8], rgb_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> usize {
    let mut pixels_ptr = pixel_data.as_ptr().cast::<[u8; PIXEL_SIZE]>();
    let mut rgbs_ptr = rgb_buf.as_mut_ptr().cast::<[u8; RGB_CHANNELS]>();
    let num_pixels = spec.num_pixels();

    for _ in 0..num_pixels {
        unsafe {
            *rgbs_ptr = pixel_to_rgb(u16::from_le_bytes(*pixels_ptr));
            pixels_ptr = pixels_ptr.add(1);
            rgbs_ptr = rgbs_ptr.add(1);
        }
    }

    if let Some(consumed_bytes) = consumed_bytes {
        *consumed_bytes = num_pixels * PIXEL_SIZE;
    }

    num_pixels * RGB_CHANNELS
}