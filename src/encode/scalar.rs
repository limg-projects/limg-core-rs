use crate::pixel::{rgb_to_pixel, ColorType, PIXEL_BYTES};
use ::core::ptr::copy_nonoverlapping;

#[inline(always)]
pub unsafe fn encode_from_rgb565_direct(data: &[u8], buf: &mut [u8], num_pixels: usize) {
    unsafe { copy_nonoverlapping(data.as_ptr(), buf.as_mut_ptr(), num_pixels * PIXEL_BYTES); }
}

#[inline(always)]
pub unsafe fn encode_from_rgb565_swap(data: &[u8], buf: &mut [u8], num_pixels: usize) {
    let mut data_ptr = data.as_ptr().cast::<u16>();
    let mut buf_ptr = buf.as_mut_ptr().cast::<u16>();

    for _ in 0..num_pixels {
        unsafe {
            let pixel = data_ptr.read_unaligned();
            buf_ptr.write_unaligned(pixel.swap_bytes());

            data_ptr = data_ptr.add(1);
            buf_ptr = buf_ptr.add(1);
        }
    }
}

macro_rules! encode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident) => {
        #[inline(always)]
        pub unsafe fn $rgb888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            const COLOR_BYTES: usize = ColorType::bytes_per_pixel(ColorType::Rgb888);
            
            let mut data_ptr = data.as_ptr().cast::<[u8; COLOR_BYTES]>();
            let mut buf_ptr = buf.as_mut_ptr().cast::<u16>();
        
            for _ in 0..num_pixels {
                unsafe {
                    let rgb = data_ptr.read();
                    let pixel = rgb_to_pixel(rgb).$endian_fn();
                    buf_ptr.write_unaligned(pixel);
        
                    data_ptr = data_ptr.add(1);
                    buf_ptr = buf_ptr.add(1);
                }
            }
        }

        #[inline(always)]
        pub unsafe fn $rgb565(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            if cfg!(target_endian = $endian) {
                unsafe { encode_from_rgb565_direct(data, buf, num_pixels); }
            } else {
                unsafe { encode_from_rgb565_swap(data, buf, num_pixels); }
            }
        }

        #[inline(always)]
        pub unsafe fn $rgba8888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            const COLOR_BYTES: usize = ColorType::bytes_per_pixel(ColorType::Rgba8888);

            let mut data_ptr = data.as_ptr().cast::<[u8; COLOR_BYTES]>();
            let mut buf_ptr = buf.as_mut_ptr().cast::<u16>();
        
            for _ in 0..num_pixels {
                unsafe {
                    let rgba = data_ptr.read();
                    let pixel = rgb_to_pixel([rgba[0], rgba[1], rgba[2]]).$endian_fn();
                    buf_ptr.write_unaligned(pixel);
        
                    data_ptr = data_ptr.add(1);
                    buf_ptr = buf_ptr.add(1);
                }
            }
        }
    };
}

encode_from_endian!("big", to_be, encode_from_rgb888_be, encode_from_rgb565_be, encode_from_rgba8888_be);
encode_from_endian!("little", to_le, encode_from_rgb888_le, encode_from_rgb565_le, encode_from_rgba8888_le);
