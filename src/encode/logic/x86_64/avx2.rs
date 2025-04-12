#![allow(unsafe_op_in_unsafe_fn)]

use crate::pixel::{ColorType, PIXEL_BYTES};
use crate::encode::logic::scalar;
use crate::common::logic::x86_64::M256I;

const PIXEL_BLOCK_LEN: usize = 16; // u16(16 bit) * 16 = 256 bit

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn encode_from_rgb565_swap(data: *const u8, buf: *mut u8, num_pixels: usize) {
    let mut data = data;
    let mut buf = buf;
    
    let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
    let remainder = num_pixels % PIXEL_BLOCK_LEN;

    for _ in 0..pixel_blocks {
        let pixel = M256I::loadu_si256(data.cast::<M256I>()).swap_epi16();

        pixel.storeu_si256(buf.cast::<M256I>());
        
        data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
        buf = buf.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
    }

    scalar::encode_from_rgb565_swap(data, buf, remainder)
}

macro_rules! encode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident) => {

        // -- rgb888 ----------------------------

        #[inline]
        #[target_feature(enable = "avx2")]
        pub unsafe fn $rgb888(data: *const u8, buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;

            const R_MASK_1: M256I = unsafe { M256I::const_i8::<
                4, -1, 7, -1, 10, -1, 13, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                0, -1, 3, -1,  6, -1,  9, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            >() };

            const G_MASK_1: M256I = unsafe { M256I::const_i8::<
                5, -1, 8, -1, 11, -1, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                1, -1, 4, -1,  7, -1, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            >() };

            const B_MASK_1: M256I = unsafe { M256I::const_i8::<
                6, -1, 9, -1, 12, -1, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                2, -1, 5, -1,  8, -1, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            >() };

            const R_MASK_2: M256I = unsafe { M256I::const_i8::<
                -1, -1, -1, -1, -1, -1, -1, -1, 4, -1, 7, -1, 10, -1, 13, -1,
                -1, -1, -1, -1, -1, -1, -1, -1, 0, -1, 3, -1,  6, -1,  9, -1,
            >() };

            const G_MASK_2: M256I = unsafe { M256I::const_i8::<
                -1, -1, -1, -1, -1, -1, -1, -1, 5, -1, 8, -1, 11, -1, 14, -1,
                -1, -1, -1, -1, -1, -1, -1, -1, 1, -1, 4, -1,  7, -1, 10, -1,
            >() };

            const B_MASK_2: M256I = unsafe { M256I::const_i8::<
                -1, -1, -1, -1, -1, -1, -1, -1, 6, -1, 9, -1, 12, -1, 15, -1,
                -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 5, -1,  8, -1, 11, -1,
            >() };

            const PERMUTE_IMM8: i32 = 0b_11_01_10_00;

            // バッファオーバーしないための前後ピクセルを加味する
            if num_pixels < PIXEL_BLOCK_LEN + 2 + 2 {
                scalar::$rgb888(data, buf, num_pixels);
            }

            // 先頭の2ピクセル先に処理する
            scalar::$rgb888(data, buf, 2);
            let num_pixels = num_pixels - 2;

            // 後半処理のため2バイトずらす
            let mut data = data.add(2);
            // 2ピクセル部分進めておく
            let mut buf = buf.add(PIXEL_BYTES * 2);

            
            let pixel_blocks = (num_pixels - 2) / PIXEL_BLOCK_LEN;
            let remainder = num_pixels - (PIXEL_BLOCK_LEN * pixel_blocks);
        
            for _ in 0..pixel_blocks {
                // 前半8ピクセル取得
                let rgb_1 = M256I::loadu_si256(data.cast::<M256I>());
                data = data.add(8 * COLOR_TYPE.bytes_per_pixel() ); 
                // 後半8ピクセル取得
                let rgb_2 = M256I::loadu_si256(data.cast::<M256I>());

                // RGBに分離
                let mut r_pixel = rgb_1.shuffle_epi8(R_MASK_1) | rgb_2.shuffle_epi8(R_MASK_2);
                let mut g_pixel = rgb_1.shuffle_epi8(G_MASK_1) | rgb_2.shuffle_epi8(G_MASK_2);
                let mut b_pixel = rgb_1.shuffle_epi8(B_MASK_1) | rgb_2.shuffle_epi8(B_MASK_2);

                // 正しい順序に並び替え
                r_pixel = r_pixel.permute4x64_epi64::<PERMUTE_IMM8>();
                g_pixel = g_pixel.permute4x64_epi64::<PERMUTE_IMM8>();
                b_pixel = b_pixel.permute4x64_epi64::<PERMUTE_IMM8>();

                // 減色 + 位置調整
                r_pixel = r_pixel.srli_epi16::<3>().slli_epi16::<11>(); // (R >> 3) << 11
                g_pixel = g_pixel.srli_epi16::<2>().slli_epi16::<5>();  // (G >> 2) << 5
                b_pixel = b_pixel.srli_epi16::<3>();                    // (B >> 3)

                // ピクセルに合成
                let pixel = (r_pixel | g_pixel | b_pixel).$endian_fn();
    
                pixel.storeu_si256(buf.cast::<M256I>());
                
                data = data.add(8 * COLOR_TYPE.bytes_per_pixel());
                buf = buf.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
            }
        
            scalar::$rgb888(data.add(4), buf, remainder)
        }

        // -- rgb565 ------------------------------

        #[inline(always)]
        #[cfg(target_endian = $endian)]
        pub unsafe fn $rgb565(data: *const u8, buf: *mut u8, num_pixels: usize) {
            scalar::encode_from_rgb565_direct(data, buf, num_pixels)
        }
        
        #[inline]
        #[target_feature(enable = "avx2")]
        #[cfg(not(target_endian = $endian))]
        pub unsafe fn $rgb565(data: *const u8, buf: *mut u8, num_pixels: usize) {
            encode_from_rgb565_swap(data, buf, num_pixels)
        }

        // -- rgba8888 ----------------------------

        #[inline]
        #[target_feature(enable = "avx2")]
        pub unsafe fn $rgba8888(data: *const u8, buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;

            const R_MASK_1: M256I = unsafe { M256I::const_i8::<
                0, -1, 4, -1, 8, -1, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                0, -1, 4, -1, 8, -1, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            >() };

            const G_MASK_1: M256I = unsafe { M256I::const_i8::<
                1, -1, 5, -1, 9, -1, 13, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                1, -1, 5, -1, 9, -1, 13, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            >() };

            const B_MASK_1: M256I = unsafe { M256I::const_i8::<
                2, -1, 6, -1, 10, -1, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                2, -1, 6, -1, 10, -1, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            >() };

            const R_MASK_2: M256I = unsafe { M256I::const_i8::<
                -1, -1, -1, -1, -1, -1, -1, -1, 0, -1, 4, -1, 8, -1, 12, -1,
                -1, -1, -1, -1, -1, -1, -1, -1, 0, -1, 4, -1, 8, -1, 12, -1,
            >() };

            const G_MASK_2: M256I = unsafe { M256I::const_i8::<
                -1, -1, -1, -1, -1, -1, -1, -1, 1, -1, 5, -1, 9, -1, 13, -1,
                -1, -1, -1, -1, -1, -1, -1, -1, 1, -1, 5, -1, 9, -1, 13, -1,
            >() };

            const B_MASK_2: M256I = unsafe { M256I::const_i8::<
                -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 6, -1, 10, -1, 14, -1,
                -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 6, -1, 10, -1, 14, -1,
            >() };

            const PERMUTE_IMM8: i32 = 0b_11_01_10_00;
        
            let mut data = data;
            let mut buf = buf;
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            for _ in 0..pixel_blocks {
                // 前半8ピクセル取得
                let rgb_1 = M256I::loadu_si256(data.cast::<M256I>());
                data = data.add(8 * COLOR_TYPE.bytes_per_pixel());
                // 後半8ピクセル取得
                let rgb_2 = M256I::loadu_si256(data.cast::<M256I>());

                // RGBに分離
                let mut r_pixel = rgb_1.shuffle_epi8(R_MASK_1) | rgb_2.shuffle_epi8(R_MASK_2);
                let mut g_pixel = rgb_1.shuffle_epi8(G_MASK_1) | rgb_2.shuffle_epi8(G_MASK_2);
                let mut b_pixel = rgb_1.shuffle_epi8(B_MASK_1) | rgb_2.shuffle_epi8(B_MASK_2);

                // 正しい順序に並び替え
                r_pixel = r_pixel.permute4x64_epi64::<PERMUTE_IMM8>();
                g_pixel = g_pixel.permute4x64_epi64::<PERMUTE_IMM8>();
                b_pixel = b_pixel.permute4x64_epi64::<PERMUTE_IMM8>();

                // 減色 + 位置調整
                r_pixel = r_pixel.srli_epi16::<3>().slli_epi16::<11>(); // (R >> 3) << 11
                g_pixel = g_pixel.srli_epi16::<2>().slli_epi16::<5>();  // (G >> 2) << 5
                b_pixel = b_pixel.srli_epi16::<3>();                    // (B >> 3)

                // ピクセルに合成
                let pixel = (r_pixel | g_pixel | b_pixel).$endian_fn();

                pixel.storeu_si256(buf.cast::<M256I>());
                
                data = data.add(8 * COLOR_TYPE.bytes_per_pixel());
                buf = buf.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
            }
        
            scalar::$rgba8888(data, buf, remainder)
        }
    };
}

encode_from_endian!("big", be_epi16, encode_from_rgb888_be, encode_from_rgb565_be, encode_from_rgba8888_be);
encode_from_endian!("little", le_epi16, encode_from_rgb888_le, encode_from_rgb565_le, encode_from_rgba8888_le);

#[cfg(test)]
mod tests {
    use crate::encode::logic::scalar;
    use crate::pixel::PIXEL_BYTES;

    use crate::encode::logic::tests::{NUM_PIXELS, RGB888_DATA, RGB565_DATA, RGBA8888_DATA};

    #[test]
    fn encode_rgb888_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut simd_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        unsafe {
            scalar::encode_from_rgb888_be(RGB888_DATA.as_ptr(), scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgb888_be(RGB888_DATA.as_ptr(), simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::encode_from_rgb888_le(RGB888_DATA.as_ptr(), scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgb888_le(RGB888_DATA.as_ptr(), simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn encode_rgb565_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut simd_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let data_ptr = (&RGB565_DATA as *const u16).cast::<u8>();
        let data = unsafe { ::core::slice::from_raw_parts(data_ptr, NUM_PIXELS * PIXEL_BYTES) };

        unsafe {
            scalar::encode_from_rgb565_be(data.as_ptr(), scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgb565_be(data.as_ptr(), simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::encode_from_rgb565_le(data.as_ptr(), scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgb565_le(data.as_ptr(), simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn encode_rgba8888_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut simd_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        unsafe {
            scalar::encode_from_rgba8888_be(RGBA8888_DATA.as_ptr(), scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgba8888_be(RGBA8888_DATA.as_ptr(), simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::encode_from_rgba8888_le(RGBA8888_DATA.as_ptr(), scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgba8888_le(RGBA8888_DATA.as_ptr(), simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn encode_endian_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let mut a_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut b_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let data_ptr = (&RGB565_DATA as *const u16).cast::<u8>();
        let data = unsafe { ::core::slice::from_raw_parts(data_ptr, NUM_PIXELS * PIXEL_BYTES) };

        unsafe {
            super::encode_from_rgb888_be(RGB888_DATA.as_ptr(), a_buf.as_mut_ptr(), NUM_PIXELS);

            super::encode_from_rgb565_be(data.as_ptr(), b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
            super::encode_from_rgba8888_be(RGBA8888_DATA.as_ptr(), b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            
            super::encode_from_rgb888_le(RGB888_DATA.as_ptr(), a_buf.as_mut_ptr(), NUM_PIXELS);

            super::encode_from_rgb565_le(data.as_ptr(), b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
            super::encode_from_rgba8888_le(RGBA8888_DATA.as_ptr(), b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
        }
    }
}