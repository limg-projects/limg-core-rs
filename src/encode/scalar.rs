use crate::pixel::{rgb_to_pixel, PIXEL_BYTES};
use ::core::ptr::{copy_nonoverlapping, read_unaligned, write_unaligned};

macro_rules! encode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident) => {
        #[inline(always)]
        pub unsafe fn $rgb888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            let mut data_ptr = data.as_ptr().cast::<[u8; 3]>();
            let mut buf_ptr = buf.as_mut_ptr().cast::<u16>();
        
            for _ in 0..num_pixels {
                unsafe {
                    let pixel = rgb_to_pixel(*data_ptr).$endian_fn();
                    write_unaligned(buf_ptr, pixel);
        
                    data_ptr = data_ptr.add(1);
                    buf_ptr = buf_ptr.add(1);
                }
            }
        }

        #[inline(always)]
        pub unsafe fn $rgb565(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            if cfg!(target_endian = $endian) {
                unsafe { copy_nonoverlapping(data.as_ptr(), buf.as_mut_ptr(), num_pixels * PIXEL_BYTES); }
            } else {
                let mut data_ptr = data.as_ptr().cast::<u16>();
                let mut buf_ptr = buf.as_mut_ptr().cast::<u16>();
        
                for _ in 0..num_pixels {
                    unsafe {
                        let pixel = read_unaligned(data_ptr);
                        write_unaligned(buf_ptr, pixel.swap_bytes());
        
                        data_ptr = data_ptr.add(1);
                        buf_ptr = buf_ptr.add(1);
                    }
                }
            }
        }

        #[inline(always)]
        pub unsafe fn $rgba8888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            let mut data_ptr = data.as_ptr().cast::<[u8; 4]>();
            let mut buf_ptr = buf.as_mut_ptr().cast::<u16>();
        
            for _ in 0..num_pixels {
                unsafe {
                    let rgba = *data_ptr;
                    let pixel = rgb_to_pixel([rgba[0], rgba[1], rgba[2]]).$endian_fn();
                    write_unaligned(buf_ptr, pixel);
        
                    data_ptr = data_ptr.add(1);
                    buf_ptr = buf_ptr.add(1);
                }
            }
        }
    };
}

encode_from_endian!("big", to_be, encode_from_rgb888_be, encode_from_rgb565_be, encode_from_rgba8888_be);
encode_from_endian!("little", to_le, encode_from_rgb888_le, encode_from_rgb565_le, encode_from_rgba8888_le);
