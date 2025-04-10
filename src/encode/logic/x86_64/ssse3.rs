#![allow(unsafe_op_in_unsafe_fn)]

use crate::{pixel::PIXEL_BYTES, ColorType};
use core::slice::{from_raw_parts, from_raw_parts_mut};
use crate::encode::logic::scalar;
use crate::common::logic::x86_64::M128I;

const PIXEL_BLOCK_LEN: usize = 8; // u16(16 bit) * 8 = 128 bit

#[inline]
#[target_feature(enable = "ssse3")]
pub unsafe fn encode_from_rgb565_swap(data: &[u8], buf: &mut [u8], num_pixels: usize) {
    let mut src_ptr = data.as_ptr();
    let mut dst_ptr = buf.as_mut_ptr();
    
    let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
    let remainder = num_pixels % PIXEL_BLOCK_LEN;

    for _ in 0..pixel_blocks {
        let src = M128I::loadu_si128(src_ptr.cast::<M128I>()).swap_epi16();

        src.storeu_si128(dst_ptr.cast::<M128I>());
        
        src_ptr = src_ptr.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
        dst_ptr = dst_ptr.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
    }

    let data = unsafe { from_raw_parts(src_ptr, remainder * PIXEL_BYTES) };
    let buf = unsafe { from_raw_parts_mut(dst_ptr, remainder * PIXEL_BYTES) };

    unsafe { scalar::encode_from_rgb565_swap(data, buf, remainder) }
}

macro_rules! encode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident) => {
        // -- rgb888 ------------------------------

        #[inline]
        #[target_feature(enable = "ssse3")]
        pub unsafe fn $rgb888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;
        
            const R_MASK_1: M128I = unsafe { M128I::new_i8(0, -1, 3, -1, 6, -1, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1) };
            const G_MASK_1: M128I = unsafe { M128I::new_i8(1, -1, 4, -1, 7, -1, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1) };
            const B_MASK_1: M128I = unsafe { M128I::new_i8(2, -1, 5, -1, 8, -1, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1) };
        
            const R_MASK_2: M128I = unsafe { M128I::new_i8(-1, -1, -1, -1, -1, -1, -1, -1, 0, -1, 3, -1, 6, -1, 9, -1) };
            const G_MASK_2: M128I = unsafe { M128I::new_i8(-1, -1, -1, -1, -1, -1, -1, -1, 1, -1, 4, -1, 7, -1, 10, -1) };
            const B_MASK_2: M128I = unsafe { M128I::new_i8(-1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 5, -1, 8, -1, 11, -1) };

            // バッファオーバーしないための後ピクセルを加味する
            if num_pixels < PIXEL_BLOCK_LEN + 2 {
                return unsafe { scalar::$rgb888(data, buf, num_pixels) };
            }
        
            let mut src_ptr = data.as_ptr();
            let mut dst_ptr = buf.as_mut_ptr();
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            for _ in 0..pixel_blocks {
                unsafe {
                    // 前半4ピクセル取得
                    let rgb_1 = M128I::loadu_si128(src_ptr.cast::<M128I>());
                    src_ptr = src_ptr.add(4 * COLOR_TYPE.bytes_per_pixel());
                    // 後半4ピクセル取得
                    let rgb_2 = M128I::loadu_si128(src_ptr.cast::<M128I>());

                    // RGBを分離
                    let mut r_pixel = rgb_1.shuffle_epi8(R_MASK_1) | rgb_2.shuffle_epi8(R_MASK_2);
                    let mut g_pixel = rgb_1.shuffle_epi8(G_MASK_1) | rgb_2.shuffle_epi8(G_MASK_2);
                    let mut b_pixel = rgb_1.shuffle_epi8(B_MASK_1) | rgb_2.shuffle_epi8(B_MASK_2);
        
                    // 減色 + 位置調整
                    r_pixel = r_pixel.srli_epi16::<3>().slli_epi16::<11>(); // (R >> 3) << 11
                    g_pixel = g_pixel.srli_epi16::<2>().slli_epi16::<5>();  // (G >> 2) << 5
                    b_pixel = b_pixel.srli_epi16::<3>();                    // (B >> 3)
        
                    // ピクセルに合成
                    let pixel = (r_pixel | g_pixel | b_pixel).$endian_fn();

                    pixel.storeu_si128(dst_ptr.cast::<M128I>());
                    
                    src_ptr = src_ptr.add(4 * COLOR_TYPE.bytes_per_pixel());
                    dst_ptr = dst_ptr.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                }
            }
        
            let data = unsafe { from_raw_parts(src_ptr, remainder * COLOR_TYPE.bytes_per_pixel()) };
            let buf = unsafe { from_raw_parts_mut(dst_ptr, remainder * PIXEL_BYTES) };
        
            unsafe { scalar::$rgb888(data, buf, remainder) }
        }

        // -- rgb565 ------------------------------

        #[inline(always)]
        #[cfg(target_endian = $endian)]
        pub unsafe fn $rgb565(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            unsafe { scalar::encode_from_rgb565_direct(data, buf, num_pixels) }
        }
        
        #[inline]
        #[target_feature(enable = "ssse3")]
        #[cfg(not(target_endian = $endian))]
        pub unsafe fn $rgb565(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            unsafe { encode_from_rgb565_swap(data, buf, num_pixels) }
        }

        // -- rgba8888 ----------------------------

        #[inline]
        #[target_feature(enable = "ssse3")]
        pub unsafe fn $rgba8888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;
        
            const R_MASK_1: M128I = unsafe { M128I::new_i8(0, -1, 4, -1, 8, -1, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1) };
            const G_MASK_1: M128I = unsafe { M128I::new_i8(1, -1, 5, -1, 9, -1, 13, -1, -1, -1, -1, -1, -1, -1, -1, -1) };
            const B_MASK_1: M128I = unsafe { M128I::new_i8(2, -1, 6, -1, 10, -1, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1) };
        
            const R_MASK_2: M128I = unsafe { M128I::new_i8(-1, -1, -1, -1, -1, -1, -1, -1, 0, -1, 4, -1, 8, -1, 12, -1) };
            const G_MASK_2: M128I = unsafe { M128I::new_i8(-1, -1, -1, -1, -1, -1, -1, -1, 1, -1, 5, -1, 9, -1, 13, -1) };
            const B_MASK_2: M128I = unsafe { M128I::new_i8(-1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 6, -1, 10, -1, 14, -1) };
        
            let mut src_ptr = data.as_ptr();
            let mut dst_ptr = buf.as_mut_ptr();
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            for _ in 0..pixel_blocks {
                unsafe {
                    // 前半4ピクセル取得
                    let rgb_1 = M128I::loadu_si128(src_ptr.cast::<M128I>());
                    src_ptr = src_ptr.add(4 * COLOR_TYPE.bytes_per_pixel());
                    // 後半4ピクセル取得
                    let rgb_2 = M128I::loadu_si128(src_ptr.cast::<M128I>());

                    // RGBに分離
                    let mut r_pixel = rgb_1.shuffle_epi8(R_MASK_1) | rgb_2.shuffle_epi8(R_MASK_2);
                    let mut g_pixel = rgb_1.shuffle_epi8(G_MASK_1) | rgb_2.shuffle_epi8(G_MASK_2);
                    let mut b_pixel = rgb_1.shuffle_epi8(B_MASK_1) | rgb_2.shuffle_epi8(B_MASK_2);

                    // 減色 + 位置調整
                    r_pixel = r_pixel.srli_epi16::<3>().slli_epi16::<11>(); // (R >> 3) << 11
                    g_pixel = g_pixel.srli_epi16::<2>().slli_epi16::<5>();  // (G >> 2) << 5
                    b_pixel = b_pixel.srli_epi16::<3>();                    // (B >> 3)

                    // ピクセルに合成
                    let pixel = (r_pixel | g_pixel | b_pixel).$endian_fn();

                    pixel.storeu_si128(dst_ptr.cast::<M128I>());
                    
                    src_ptr = src_ptr.add(4 * COLOR_TYPE.bytes_per_pixel());
                    dst_ptr = dst_ptr.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                }
            }
        
            let data = unsafe { from_raw_parts(src_ptr, remainder * COLOR_TYPE.bytes_per_pixel()) };
            let buf = unsafe { from_raw_parts_mut(dst_ptr, remainder * PIXEL_BYTES) };
        
            unsafe { scalar::$rgba8888(data, buf, remainder) }
        }
    };
}

encode_from_endian!("big", be_epi16, encode_from_rgb888_be, encode_from_rgb565_be, encode_from_rgba8888_be);
encode_from_endian!("little", le_epi16, encode_from_rgb888_le, encode_from_rgb565_le, encode_from_rgba8888_le);

#[cfg(test)]
mod tests {
    use crate::encode::logic::scalar;
    use crate::PIXEL_BYTES;

    use crate::encode::logic::tests::{NUM_PIXELS, RGB888_DATA, RGB565_DATA, RGBA8888_DATA};

    #[test]
    fn encode_rgb888_x86_64_sse41() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }
        
        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut sse41_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        unsafe {
            scalar::encode_from_rgb888_be(&RGB888_DATA, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgb888_be(&RGB888_DATA, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);

        unsafe {
            scalar::encode_from_rgb888_le(&RGB888_DATA, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgb888_le(&RGB888_DATA, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);
    }

    #[test]
    fn encode_rgb565_x86_64_sse41() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut sse41_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let data_ptr = (&RGB565_DATA as *const u16).cast::<u8>();
        let data = unsafe { ::core::slice::from_raw_parts(data_ptr, NUM_PIXELS * PIXEL_BYTES) };

        unsafe {
            scalar::encode_from_rgb565_be(data, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgb565_be(data, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);

        unsafe {
            scalar::encode_from_rgb565_le(data, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgb565_le(data, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);
    }

    #[test]
    fn encode_rgba8888_x86_64_sse41() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut sse41_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        unsafe {
            scalar::encode_from_rgba8888_be(&RGBA8888_DATA, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgba8888_be(&RGBA8888_DATA, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);

        unsafe {
            scalar::encode_from_rgba8888_le(&RGBA8888_DATA, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgba8888_le(&RGBA8888_DATA, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);
    }

    #[test]
    fn encode_endian_x86_64_sse41() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut a_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut b_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let data_ptr = (&RGB565_DATA as *const u16).cast::<u8>();
        let data = unsafe { ::core::slice::from_raw_parts(data_ptr, NUM_PIXELS * PIXEL_BYTES) };

        unsafe {
            super::encode_from_rgb888_be(&RGB888_DATA, &mut a_buf, NUM_PIXELS);

            super::encode_from_rgb565_be(data, &mut b_buf, NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
            super::encode_from_rgba8888_be(&RGBA8888_DATA, &mut b_buf, NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            
            super::encode_from_rgb888_le(&RGB888_DATA, &mut a_buf, NUM_PIXELS);

            super::encode_from_rgb565_le(data, &mut b_buf, NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
            super::encode_from_rgba8888_le(&RGBA8888_DATA, &mut b_buf, NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
        }
    }
}