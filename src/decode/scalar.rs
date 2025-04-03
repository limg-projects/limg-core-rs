use core::ptr::copy_nonoverlapping;
use crate::{pixel_to_rgb, ColorType, PIXEL_BYTES};


pub unsafe fn decode_to_rgb888_be(data: &[u8], buf: &mut [u8], num_pixels: usize) {
    const COLOR_BYTES: usize = ColorType::bytes_per_pixel(ColorType::Rgb888);

    let mut data_ptr = data.as_ptr().cast::<u16>();
    let mut buf_ptr = buf.as_mut_ptr().cast::<[u8; COLOR_BYTES]>();

    for _ in 0..num_pixels {
        unsafe {
            let pixel = data_ptr.read_unaligned().to_be();
            buf_ptr.write(pixel_to_rgb(pixel));

            data_ptr = data_ptr.add(1);
            buf_ptr = buf_ptr.add(1);
        }
    }
}

pub unsafe fn decode_to_rgb888_le(data: &[u8], buf: &mut [u8], num_pixels: usize) {

}

pub unsafe fn decode_to_rgb565_be(data: &[u8], buf: &mut [u8], num_pixels: usize) {
    if cfg!(target_endian = "big") {
        unsafe { copy_nonoverlapping(data.as_ptr(), buf.as_mut_ptr(), num_pixels * PIXEL_BYTES); }
    } else {
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
}

pub unsafe fn decode_to_rgb565_le(data: &[u8], buf: &mut [u8], num_pixels: usize) {

}

pub unsafe fn decode_to_rgba8888_be(data: &[u8], buf: &mut [u8], num_pixels: usize) {
    const COLOR_BYTES: usize = ColorType::bytes_per_pixel(ColorType::Rgba8888);

    let mut data_ptr = data.as_ptr().cast::<u16>();
    let mut buf_ptr = buf.as_mut_ptr().cast::<[u8; COLOR_BYTES]>();

    for _ in 0..num_pixels {
        unsafe {
            let pixel = data_ptr.read_unaligned().to_be();
            let rgb = pixel_to_rgb(pixel);
            buf_ptr.write([rgb[0], rgb[1], rgb[2], 0]);

            data_ptr = data_ptr.add(1);
            buf_ptr = buf_ptr.add(1);
        }
    }
}

pub unsafe fn decode_to_rgba8888_le(data: &[u8], buf: &mut [u8], num_pixels: usize) {

}