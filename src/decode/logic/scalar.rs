use crate::decode_logic_fn;
use crate::pixel::{pixel_to_rgb, ColorType, PIXEL_BYTES};

#[inline(always)]
const fn pixel_to_alpha(pixel: u16, transparent_color: u16) -> u8 {
    0u8.wrapping_sub((pixel != transparent_color) as u8)
}

#[inline(always)]
pub unsafe fn decode_from_rgb565_direct(data: *const u8, buf: *mut u8, num_pixels: usize) {
    unsafe { ::core::ptr::copy_nonoverlapping(data, buf, num_pixels * PIXEL_BYTES); }
}

#[inline(always)]
pub unsafe fn decode_from_rgb565_swap(data: *const u8, buf: *mut u8, num_pixels: usize) {
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

macro_rules! decode_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident, $rgba8888_alpha: ident) => {
        #[inline(always)]
        pub unsafe fn $rgb888(data: *const u8, buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;

            let mut data = data.cast::<u16>();
            let mut buf = buf.cast::<[u8; COLOR_TYPE.bytes_per_pixel()]>();
        
            for _ in 0..num_pixels {
                unsafe {
                    let pixel = data.read_unaligned().$endian_fn();
                    buf.write(pixel_to_rgb(pixel));
        
                    data = data.add(1);
                    buf = buf.add(1);
                }
            }
        }

        #[inline(always)]
        pub unsafe fn $rgb565(data: *const u8, buf: *mut u8, num_pixels: usize) {
            if cfg!(target_endian = $endian) {
                unsafe { decode_from_rgb565_direct(data, buf, num_pixels); }
            } else {
                unsafe { decode_from_rgb565_swap(data, buf, num_pixels); }
            }
        }

        #[inline(always)]
        pub unsafe fn $rgba8888(data: *const u8, buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;

            let mut data = data.cast::<u16>();
            let mut buf = buf.cast::<[u8; COLOR_TYPE.bytes_per_pixel()]>();
        
            for _ in 0..num_pixels {
                unsafe {
                    let pixel = data.read_unaligned().$endian_fn();
                    let rgb = pixel_to_rgb(pixel);
                    buf.write([rgb[0], rgb[1], rgb[2], u8::MAX]);
        
                    data = data.add(1);
                    buf = buf.add(1);
                }
            }
        }

        #[inline(always)]
        pub unsafe fn $rgba8888_alpha(data: *const u8, buf: *mut u8, transparent_color: u16, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;
        
            let mut data = data.cast::<u16>();
            let mut buf = buf.cast::<[u8; COLOR_TYPE.bytes_per_pixel()]>();
        
            for _ in 0..num_pixels {
                unsafe {
                    let pixel = data.read_unaligned().$endian_fn();
                    let rgb = pixel_to_rgb(pixel);
                    let alpha = pixel_to_alpha(pixel, transparent_color);
                    buf.write([rgb[0], rgb[1], rgb[2], alpha]);
        
                    data = data.add(1);
                    buf = buf.add(1);
                }
            }
        }
    };
}

decode_logic_fn!();

decode_endian!(
    "big",
    to_be,
    decode_to_rgb888_be,
    decode_to_rgb565_be,
    decode_to_rgba8888_be,
    decode_to_rgba8888_alpha_be
);

decode_endian!(
    "little",
    to_le,
    decode_to_rgb888_le,
    decode_to_rgb565_le,
    decode_to_rgba8888_le,
    decode_to_rgba8888_alpha_le
);
