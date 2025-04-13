use crate::pixel::{rgb_to_pixel, ColorType, PIXEL_BYTES};
use crate::encode::logic::encode_logic_fn;

#[inline(always)]
pub unsafe fn encode_from_rgb565_direct(data: *const u8, buf: *mut u8, num_pixels: usize) {
    unsafe { ::core::ptr::copy_nonoverlapping(data, buf, num_pixels * PIXEL_BYTES); }
}

#[inline(always)]
pub unsafe fn encode_from_rgb565_swap(data: *const u8, buf: *mut u8, num_pixels: usize) {
    let mut data = data.cast::<u16>();
    let mut buf = buf.cast::<u16>();

    for _ in 0..num_pixels {
        unsafe {
            let pixel = data.read_unaligned();
            buf.write_unaligned(pixel.swap_bytes());

            data = data.add(1);
            buf = buf.add(1);
        }
    }
}

macro_rules! encode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident) => {
        #[inline(always)]
        pub unsafe fn $rgb888(data: *const u8, buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;
            
            let mut data = data.cast::<[u8; COLOR_TYPE.bytes_per_pixel()]>();
            let mut buf = buf.cast::<u16>();
        
            for _ in 0..num_pixels {
                unsafe {
                    let rgb = data.read();
                    let pixel = rgb_to_pixel(rgb).$endian_fn();
                    buf.write_unaligned(pixel);
        
                    data = data.add(1);
                    buf = buf.add(1);
                }
            }
        }

        #[inline(always)]
        pub unsafe fn $rgb565(data: *const u8, buf: *mut u8, num_pixels: usize) {
            if cfg!(target_endian = $endian) {
                unsafe { encode_from_rgb565_direct(data, buf, num_pixels); }
            } else {
                unsafe { encode_from_rgb565_swap(data, buf, num_pixels); }
            }
        }

        #[inline(always)]
        pub unsafe fn $rgba8888(data: *const u8, buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;

            let mut data = data.cast::<[u8; COLOR_TYPE.bytes_per_pixel()]>();
            let mut buf = buf.cast::<u16>();
        
            for _ in 0..num_pixels {
                unsafe {
                    let rgba = data.read();
                    let pixel = rgb_to_pixel([rgba[0], rgba[1], rgba[2]]).$endian_fn();
                    buf.write_unaligned(pixel);
        
                    data = data.add(1);
                    buf = buf.add(1);
                }
            }
        }
    };
}

encode_logic_fn!();
encode_from_endian!("big", to_be, encode_from_rgb888_be, encode_from_rgb565_be, encode_from_rgba8888_be);
encode_from_endian!("little", to_le, encode_from_rgb888_le, encode_from_rgb565_le, encode_from_rgba8888_le);

#[cfg(test)]
mod tests {
    use crate::pixel::PIXEL_BYTES;
    use crate::encode::logic::tests::{NUM_PIXELS, RGB888_DATA, RGB565_DATA, RGBA8888_DATA};

    #[test]
    fn encode_endian_scalar() {
        let mut a_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut b_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let rgb888_ptr = RGB888_DATA.as_ptr();
        let rgb565_ptr = RGB565_DATA.as_ptr().cast::<u8>();
        let rgba8888_ptr  = RGBA8888_DATA.as_ptr();

        unsafe {
            super::encode_from_rgb888_be(rgb888_ptr, a_buf.as_mut_ptr(), NUM_PIXELS);

            super::encode_from_rgb565_be(rgb565_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
            super::encode_from_rgba8888_be(rgba8888_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            
            super::encode_from_rgb888_le(rgb888_ptr, a_buf.as_mut_ptr(), NUM_PIXELS);

            super::encode_from_rgb565_le(rgb565_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
            super::encode_from_rgba8888_le(rgba8888_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
        }
    }
}