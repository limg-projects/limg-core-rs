use crate::{rgb_to_pixel, Error, ImageHeaderInternal, ImageSpec, Result, IMAGE_SIGNATURE_U32_NE, IMAGE_HEADER_SIZE, PIXEL_SIZE, RGB_CHANNELS};

#[inline(always)]
pub const fn encode_bounds(spec: &ImageSpec) -> usize {
    IMAGE_HEADER_SIZE + spec.num_pixels() * PIXEL_SIZE
}

#[inline]
pub fn encode(rgb_data: &[u8], image_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }

    if image_buf.len() < encode_bounds(spec) {
        return Err(Error::OutputBufferTooSmall);
    }

    if rgb_data.len() < RGB_CHANNELS * spec.num_pixels() {
        return Err(Error::InputBufferTooSmall);
    }

    unsafe {
        Ok(encode_unchecked(rgb_data, image_buf, spec, consumed_bytes))
    }
}

#[inline]
pub unsafe fn encode_unchecked(rgb_data: &[u8], image_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> usize {
    let mut written_bytes = 0;
    unsafe {
        let header_slice = image_buf.get_unchecked_mut(..IMAGE_HEADER_SIZE);
        written_bytes += encode_header_unchecked(header_slice, spec);

        let data_slice = image_buf.get_unchecked_mut(IMAGE_HEADER_SIZE..);
        written_bytes += encode_data_raw_unchecked(rgb_data, data_slice, spec, consumed_bytes);
    }
    written_bytes
}

#[inline]
pub fn encode_header(buf: &mut [u8], spec: &ImageSpec) -> Result<usize> {
    if spec.width == 0 || spec.height == 0 {
        return Err(Error::ZeroImageDimensions);
    }
    if buf.len() < size_of::<ImageHeaderInternal>() {
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
        ::core::ptr::copy_nonoverlapping(header_ptr, buf.as_mut_ptr(), size_of::<ImageHeaderInternal>());
    }

    size_of::<ImageHeaderInternal>()
}

#[inline]
pub fn encode_data(rgb_data: &[u8], pixel_buf: &mut [u16], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }

    let num_pixels = spec.num_pixels();

    if pixel_buf.len() < num_pixels {
        return Err(Error::OutputBufferTooSmall);
    }

    if rgb_data.len() < num_pixels * RGB_CHANNELS {
        return Err(Error::InputBufferTooSmall);
    }

    unsafe {
        Ok(encode_data_unchecked(rgb_data, pixel_buf, spec, consumed_bytes))
    }
}

#[inline]
pub unsafe fn encode_data_unchecked(rgb_data: &[u8], pixel_buf: &mut [u16], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> usize {
    unsafe {
        let pixel_buf = ::core::slice::from_raw_parts_mut(pixel_buf.as_mut_ptr() as *mut u8, pixel_buf.len() * size_of::<u16>());
        encode_data_raw_unchecked(rgb_data, pixel_buf, spec, consumed_bytes)
    }
}

#[inline]
pub fn encode_data_raw(rgb_data: &[u8], pixel_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }

    let num_pixels = spec.num_pixels();

    if pixel_buf.len() < num_pixels * size_of::<u16>() {
        return Err(Error::OutputBufferTooSmall);
    }

    if rgb_data.len() < num_pixels * RGB_CHANNELS {
        return Err(Error::InputBufferTooSmall);
    }

    unsafe {
        Ok(encode_data_raw_unchecked(rgb_data, pixel_buf, spec, consumed_bytes))
    }
}

pub unsafe fn encode_data_raw_unchecked(rgb_data: &[u8], pixel_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> usize {
    let mut rgb_ptr = rgb_data.as_ptr().cast::<[u8; 3]>();
    let mut pixel_ptr = pixel_buf.as_mut_ptr().cast::<[u8; 2]>();

    let mut written_bytes = 0;
    let mut _consumed_bytes = 0;

    for _ in 0..spec.num_pixels() {
        unsafe {
            *pixel_ptr = rgb_to_pixel(*rgb_ptr).to_le_bytes();
            rgb_ptr = rgb_ptr.add(1);
            pixel_ptr = pixel_ptr.add(1);
        }
        written_bytes += size_of::<u16>();
        _consumed_bytes += RGB_CHANNELS;
    }

    if let Some(consumed_bytes) = consumed_bytes {
        *consumed_bytes = _consumed_bytes;
    }

    written_bytes
}