
use crate::{pixel_to_rgb, Error, ImageHeaderInternal, ImageSpec, Result, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE, PIXEL_SIZE, RGB_CHANNELS };

pub fn decode(src: &[u8], dest: &mut [u8], decoded_size: Option<&mut usize>) -> Result<(ImageSpec, usize)> {
    let mut result_size = 0;
    let mut consumed_size = 0;

    let spec = decode_header(src, Some(&mut consumed_size))?;
    result_size += consumed_size;
    let written_size = decode_data_raw(src, dest, &spec, Some(&mut consumed_size))?;

    if let Some(decoded_size) = decoded_size {
        *decoded_size = result_size;
    }

    Ok((spec, written_size))
}

pub unsafe fn decode_unchecked(src: &[u8], dest: &mut [u8], decoded_size: Option<&mut usize>) -> (ImageSpec, usize) {
    let mut decoded_size = decoded_size;

    unsafe {
        let spec = decode_header_unchecked(src, decoded_size.as_mut().map(|s| &mut **s));
        let rgb_data = src.get_unchecked(IMAGE_HEADER_SIZE..);
        let size = IMAGE_HEADER_SIZE + decode_data_raw_unchecked(rgb_data, dest, &spec, decoded_size.as_mut().map(|s| &mut **s));
        (spec, size)
    }
}

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

#[inline]
pub fn decode_data(src: &[u16], dest: &mut [u8], spec: &ImageSpec, decoded_size: Option<&mut usize>) -> Result<usize> {
    let src = unsafe {
        ::core::slice::from_raw_parts(src.as_ptr().cast::<u8>(), src.len() * PIXEL_SIZE)
    };
    decode_data_raw(src, dest, spec, decoded_size)
}

#[inline]
pub unsafe fn decode_data_unchecked(src: &[u16], dest: &mut [u8], spec: &ImageSpec, decoded_size: Option<&mut usize>) -> usize {
    unsafe {
        let src = ::core::slice::from_raw_parts(src.as_ptr().cast::<u8>(), src.len() * size_of::<u16>());
        decode_data_raw_unchecked(src, dest, spec, decoded_size)
    }
}

#[inline]
pub fn decode_data_raw(src: &[u8], dest: &mut [u8], spec: &ImageSpec, decoded_size: Option<&mut usize>) -> Result<usize> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }

    let data_len = spec.num_pixels();

    if src.len() < data_len * PIXEL_SIZE {
        return Err(Error::InputBufferTooSmall);
    }
    if dest.len() < data_len * RGB_CHANNELS {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(decode_data_raw_unchecked(src, dest, spec, decoded_size))
    }
}

pub unsafe fn decode_data_raw_unchecked(src: &[u8], dest: &mut [u8], spec: &ImageSpec, decoded_size: Option<&mut usize>) -> usize {
    let mut pixels_ptr = src.as_ptr().cast::<[u8; PIXEL_SIZE]>();
    let mut rgbs_ptr = dest.as_mut_ptr().cast::<[u8; RGB_CHANNELS]>();
    let num_pixels = spec.num_pixels();

    for _ in 0..num_pixels {
        unsafe {
            *rgbs_ptr = pixel_to_rgb(u16::from_le_bytes(*pixels_ptr));
            pixels_ptr = pixels_ptr.add(1);
            rgbs_ptr = rgbs_ptr.add(1);
        }
    }

    if let Some(decoded_size) = decoded_size {
        *decoded_size = num_pixels * PIXEL_SIZE;
    }

    num_pixels * RGB_CHANNELS
}