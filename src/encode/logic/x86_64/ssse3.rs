#![allow(unsafe_op_in_unsafe_fn)]

use crate::pixel::PIXEL_BYTES;
use crate::encode::logic::{scalar, encode_logic_fn};
use crate::common::color::ColorType;
use crate::common::logic::x86_64::M128I;

const PIXEL_BLOCK_LEN: usize = 8; // u16(16 bit) * 8 = 128 bit

#[inline]
#[target_feature(enable = "ssse3")]
pub unsafe fn encode_from_rgb565_swap(data: *const u8, buf: *mut u8, num_pixels: usize) {
    let mut data = data;
    let mut buf = buf;

    let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
    let remainder = num_pixels % PIXEL_BLOCK_LEN;

    for _ in 0..pixel_blocks {
        let pixel = M128I::loadu_si128(data.cast::<M128I>()).swap_epi16();

        pixel.storeu_si128(buf.cast::<M128I>());
        
        data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
        buf = buf.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
    }

    scalar::encode_from_rgb565_swap(data, buf, remainder)
}

macro_rules! encode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident) => {
        // -- rgb888 ------------------------------

        #[inline]
        #[target_feature(enable = "ssse3")]
        pub unsafe fn $rgb888(data: *const u8, buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;
        
            const R_MASK_1: M128I = unsafe { M128I::const_i8::<0, -1, 3, -1, 6, -1,  9, -1, -1, -1, -1, -1, -1, -1, -1, -1>() };
            const G_MASK_1: M128I = unsafe { M128I::const_i8::<1, -1, 4, -1, 7, -1, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1>() };
            const B_MASK_1: M128I = unsafe { M128I::const_i8::<2, -1, 5, -1, 8, -1, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1>() };
        
            const R_MASK_2: M128I = unsafe { M128I::const_i8::<-1, -1, -1, -1, -1, -1, -1, -1, 0, -1, 3, -1, 6, -1,  9, -1>() };
            const G_MASK_2: M128I = unsafe { M128I::const_i8::<-1, -1, -1, -1, -1, -1, -1, -1, 1, -1, 4, -1, 7, -1, 10, -1>() };
            const B_MASK_2: M128I = unsafe { M128I::const_i8::<-1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 5, -1, 8, -1, 11, -1>() };

            // バッファオーバーしないための後ピクセルを加味する
            if num_pixels < PIXEL_BLOCK_LEN + 2 {
                return scalar::$rgb888(data, buf, num_pixels);
            }
        
            let mut data = data;
            let mut buf = buf;
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            for _ in 0..pixel_blocks {
                // 前半4ピクセル取得
                let rgb_1 = M128I::loadu_si128(data.cast::<M128I>());
                data = data.add(4 * COLOR_TYPE.bytes_per_pixel());
                // 後半4ピクセル取得
                let rgb_2 = M128I::loadu_si128(data.cast::<M128I>());

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

                pixel.storeu_si128(buf.cast::<M128I>());
                
                data = data.add(4 * COLOR_TYPE.bytes_per_pixel());
                buf = buf.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
            }
        
            scalar::$rgb888(data, buf, remainder)
        }

        // -- rgb565 ------------------------------

        #[inline(always)]
        #[cfg(target_endian = $endian)]
        pub unsafe fn $rgb565(data: *const u8, buf: *mut u8, num_pixels: usize) {
            scalar::encode_from_rgb565_direct(data, buf, num_pixels)
        }
        
        #[inline]
        #[target_feature(enable = "ssse3")]
        #[cfg(not(target_endian = $endian))]
        pub unsafe fn $rgb565(data: *const u8, buf: *mut u8, num_pixels: usize) {
            encode_from_rgb565_swap(data, buf, num_pixels)
        }

        // -- rgba8888 ----------------------------

        #[inline]
        #[target_feature(enable = "ssse3")]
        pub unsafe fn $rgba8888(data: *const u8, buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;
        
            const R_MASK_1: M128I = unsafe { M128I::const_i8::<0, -1, 4, -1,  8, -1, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1>() };
            const G_MASK_1: M128I = unsafe { M128I::const_i8::<1, -1, 5, -1,  9, -1, 13, -1, -1, -1, -1, -1, -1, -1, -1, -1>() };
            const B_MASK_1: M128I = unsafe { M128I::const_i8::<2, -1, 6, -1, 10, -1, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1>() };
        
            const R_MASK_2: M128I = unsafe { M128I::const_i8::<-1, -1, -1, -1, -1, -1, -1, -1, 0, -1, 4, -1,  8, -1, 12, -1>() };
            const G_MASK_2: M128I = unsafe { M128I::const_i8::<-1, -1, -1, -1, -1, -1, -1, -1, 1, -1, 5, -1,  9, -1, 13, -1>() };
            const B_MASK_2: M128I = unsafe { M128I::const_i8::<-1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 6, -1, 10, -1, 14, -1>() };
        
            let mut data = data;
            let mut buf = buf;
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            for _ in 0..pixel_blocks {
                // 前半4ピクセル取得
                let rgb_1 = M128I::loadu_si128(data.cast::<M128I>());
                data = data.add(4 * COLOR_TYPE.bytes_per_pixel());
                // 後半4ピクセル取得
                let rgb_2 = M128I::loadu_si128(data.cast::<M128I>());

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

                pixel.storeu_si128(buf.cast::<M128I>());
                
                data = data.add(4 * COLOR_TYPE.bytes_per_pixel());
                buf = buf.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
            }
                
            scalar::$rgba8888(data, buf, remainder)
        }
    };
}

encode_logic_fn!(#[target_feature(enable = "ssse3")]);
encode_from_endian!("big", be_epi16, encode_from_rgb888_be, encode_from_rgb565_be, encode_from_rgba8888_be);
encode_from_endian!("little", le_epi16, encode_from_rgb888_le, encode_from_rgb565_le, encode_from_rgba8888_le);

#[cfg(test)]
mod tests {
    use crate::common::color::ColorType;
    use crate::common::spec::{DataEndian, ImageSpec};
    use crate::pixel::PIXEL_BYTES;
    use crate::encode::logic::scalar;
    use crate::encode::logic::tests::{NUM_PIXELS, RGB888_DATA, RGB565_DATA, RGBA8888_DATA};

    #[test]
    fn encode_logic_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut a_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut b_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let rgb888_ptr = RGB888_DATA.as_ptr();
        let rgb565_ptr = RGB565_DATA.as_ptr().cast::<u8>();
        let rgba8888_ptr  = RGBA8888_DATA.as_ptr();

        let mut spec = ImageSpec::with_data_endian(NUM_PIXELS as u16, 1, DataEndian::Big);

        unsafe {
            super::encode_logic(rgb888_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgb888);
            super::encode_from_rgb888_be(rgb888_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            super::encode_logic(rgb565_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgb565);
            super::encode_from_rgb565_be(rgb565_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            super::encode_logic(rgba8888_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgba8888);
            super::encode_from_rgba8888_be(rgba8888_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            spec.data_endian = DataEndian::Little;

            super::encode_logic(rgb888_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgb888);
            super::encode_from_rgb888_le(rgb888_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            super::encode_logic(rgb565_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgb565);
            super::encode_from_rgb565_le(rgb565_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            super::encode_logic(rgba8888_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgba8888);
            super::encode_from_rgba8888_le(rgba8888_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
        }
    }

    #[test]
    fn encode_rgb888_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }
        
        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut simd_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let data_ptr = RGB888_DATA.as_ptr();

        unsafe {
            scalar::encode_from_rgb888_be(data_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgb888_be(data_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::encode_from_rgb888_le(data_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgb888_le(data_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn encode_rgb565_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut simd_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let data_ptr = RGB565_DATA.as_ptr().cast::<u8>();

        unsafe {
            scalar::encode_from_rgb565_be(data_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgb565_be(data_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::encode_from_rgb565_le(data_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgb565_le(data_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn encode_rgba8888_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut simd_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let data_ptr = RGBA8888_DATA.as_ptr();

        unsafe {
            scalar::encode_from_rgba8888_be(data_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgba8888_be(data_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::encode_from_rgba8888_le(data_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::encode_from_rgba8888_le(data_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn encode_endian_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

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