#![allow(unsafe_op_in_unsafe_fn)]

use crate::pixel::{ColorType, PIXEL_BYTES, PIXEL_R_MASK, PIXEL_G_MASK, PIXEL_B_MASK};
use crate::decode::logic::scalar;
use crate::common::logic::x86_64::M128I;

const PIXEL_BLOCK_LEN: usize = 8; // u16(16 bit) * 8 = 128 bit

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn get_rgb_vec(pixel: M128I) -> (M128I, M128I, M128I) {
    const R_MASK: M128I = unsafe { M128I::const1_u16::<PIXEL_R_MASK>() };
    const G_MASK: M128I = unsafe { M128I::const1_u16::<PIXEL_G_MASK>() };
    const B_MASK: M128I = unsafe { M128I::const1_u16::<PIXEL_B_MASK>() };

    (pixel & R_MASK, pixel & G_MASK, pixel & B_MASK)
}

#[inline]
#[target_feature(enable = "ssse3")]
pub unsafe fn decode_from_rgb565_swap(data: *const u8, buf: *mut u8, num_pixels: usize) {
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

    scalar::decode_from_rgb565_swap(data, buf, remainder)
}

macro_rules! encode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident, $rgba8888_aplha: ident) => {
        // -- rgb888 ------------------------------

        #[inline]
        #[target_feature(enable = "sse4.1")]
        pub unsafe fn $rgb888(data: *const u8, buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;

            // バッファオーバーしないための後ピクセルを加味する
            if num_pixels < PIXEL_BLOCK_LEN + 2 {
                return unsafe { scalar::$rgb888(data, buf, num_pixels) };
            }
        
            let mut data = data;
            let mut buf = buf;
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;

            // let R_PIXEL_MASK = __
        }

        // -- rgb565 ------------------------------

        #[inline(always)]
        #[cfg(target_endian = $endian)]
        pub unsafe fn $rgb565(data: *const u8, buf: *mut u8, num_pixels: usize) {
            scalar::decode_from_rgb565_direct(data, buf, num_pixels)
        }
        
        #[inline]
        #[target_feature(enable = "ssse3")]
        #[cfg(not(target_endian = $endian))]
        pub unsafe fn $rgb565(data: *const u8, buf: *mut u8, num_pixels: usize) {
            decode_from_rgb565_swap(data, buf, num_pixels)
        }

        // -- rgba8888 ----------------------------

        #[inline]
        #[target_feature(enable = "ssse3")]
        pub unsafe fn $rgba8888(data: *const u8, buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;

            let mut data = data;
            let mut buf = buf;
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            const R_MASK_1: M128I = unsafe {
                M128I::const_i8::< 0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6, -1, -1, -1>()
            };
            const G_MASK_1: M128I = unsafe {
                M128I::const_i8::<-1,  0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6, -1, -1>()
            };
            const B_MASK_1: M128I = unsafe {
                M128I::const_i8::<-1, -1,  0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6, -1>()
            };
        
            const R_MASK_2: M128I = unsafe {
                M128I::const_i8::< 8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14, -1, -1, -1>()
            };
            const G_MASK_2: M128I = unsafe {
                M128I::const_i8::<-1,  8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14, -1, -1>()
            };
            const B_MASK_2: M128I = unsafe {
                M128I::const_i8::<-1, -1,  8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14, -1>()
            };
        
            const ALPHA_VEC: M128I = unsafe {
                M128I::const_u8::<0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255>()
            };
        
            for _ in 0..pixel_blocks {
                // 8ピクセル取得
                let pixel = M128I::loadu_si128(data.cast::<M128I>()).$endian_fn();
        
                // マスクで色を分離
                let (mut r_vec, mut g_vec, mut b_vec) = get_rgb_vec(pixel);
        
                // 位置移動
                r_vec = r_vec.srli_epi16::<11>();
                g_vec = g_vec.srli_epi16::<5>();
        
                // 色戻し
                r_vec = r_vec.slli_epi16::<3>() | r_vec.srli_epi16::<2>();
                g_vec = g_vec.slli_epi16::<2>() | g_vec.srli_epi16::<4>();
                b_vec = b_vec.slli_epi16::<3>() | b_vec.srli_epi16::<2>();
        
                let rgb_1 = r_vec.shuffle_epi8(R_MASK_1) | g_vec.shuffle_epi8(G_MASK_1) | b_vec.shuffle_epi8(B_MASK_1) | ALPHA_VEC;
                let rgb_2 = r_vec.shuffle_epi8(R_MASK_2) | g_vec.shuffle_epi8(G_MASK_2) | b_vec.shuffle_epi8(B_MASK_2) | ALPHA_VEC;
        
                rgb_1.storeu_si128(buf.cast::<M128I>());
                buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());
                rgb_2.storeu_si128(buf.cast::<M128I>());
        
                data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());
            }
            
            scalar::$rgba8888(data, buf, remainder)
        }

        // -- rgba8888 alpha ----------------------

        #[inline]
        #[target_feature(enable = "ssse3")]
        pub unsafe fn $rgba8888_aplha(data: *const u8, buf: *mut u8, transparent_color: u16, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;

            let mut data = data;
            let mut buf = buf;
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            const R_MASK_1: M128I = unsafe {
                M128I::const_i8::< 0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6, -1, -1, -1>()
            };
            const G_MASK_1: M128I = unsafe {
                M128I::const_i8::<-1,  0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6, -1, -1>()
            };
            const B_MASK_1: M128I = unsafe {
                M128I::const_i8::<-1, -1,  0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6, -1>()
            };
            const A_MASK_1: M128I = unsafe {
                M128I::const_i8::<-1, -1, -1,  0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6>()
            };
        
            const R_MASK_2: M128I = unsafe {
                M128I::const_i8::< 8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14, -1, -1, -1>()
            };
            const G_MASK_2: M128I = unsafe {
                M128I::const_i8::<-1,  8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14, -1, -1>()
            };
            const B_MASK_2: M128I = unsafe {
                M128I::const_i8::<-1, -1,  8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14, -1>()
            };
            const A_MASK_2: M128I = unsafe {
                M128I::const_i8::<-1, -1, -1,  8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14>()
            };
        
            let transparent_vec = M128I::set1_epi16(transparent_color as i16);
        
            for _ in 0..pixel_blocks {
                // 8ピクセル取得
                let pixel = M128I::loadu_si128(data.cast::<M128I>()).$endian_fn();
        
                // alpha作成
                let a_vec = pixel.cmpeq_epi16(transparent_vec).not_si128().srli_epi16::<8>();
        
                // マスクで色を分離
                let (mut r_vec, mut g_vec, mut b_vec) = get_rgb_vec(pixel);
        
                // 位置移動
                r_vec = r_vec.srli_epi16::<11>();
                g_vec = g_vec.srli_epi16::<5>();
        
                // 色戻し
                r_vec = r_vec.slli_epi16::<3>() | r_vec.srli_epi16::<2>();
                g_vec = g_vec.slli_epi16::<2>() | g_vec.srli_epi16::<4>();
                b_vec = b_vec.slli_epi16::<3>() | b_vec.srli_epi16::<2>();
        
                // rgbaに合成
                let rgb_1 = r_vec.shuffle_epi8(R_MASK_1) | g_vec.shuffle_epi8(G_MASK_1) | b_vec.shuffle_epi8(B_MASK_1) | a_vec.shuffle_epi8(A_MASK_1);
                let rgb_2 = r_vec.shuffle_epi8(R_MASK_2) | g_vec.shuffle_epi8(G_MASK_2) | b_vec.shuffle_epi8(B_MASK_2) | a_vec.shuffle_epi8(A_MASK_2);
        
                rgb_1.storeu_si128(buf.cast::<M128I>());
                buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());
                rgb_2.storeu_si128(buf.cast::<M128I>());
        
                data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());
            }
            
            scalar::$rgba8888_aplha(data, buf, transparent_color, remainder)
        }
    };
}

#[target_feature(enable = "ssse3")]
unsafe fn a(data: *const u8, buf: *mut u8, transparent_color: u16, num_pixels: usize) {
    const COLOR_TYPE: ColorType = ColorType::Rgba8888;

    let mut data = data;
    let mut buf = buf;
    
    let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
    let remainder = num_pixels % PIXEL_BLOCK_LEN;

    const R_MASK_1: M128I = unsafe {
        M128I::const_i8::< 0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6, -1, -1, -1>()
    };
    const G_MASK_1: M128I = unsafe {
        M128I::const_i8::<-1,  0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6, -1, -1>()
    };
    const B_MASK_1: M128I = unsafe {
        M128I::const_i8::<-1, -1,  0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6, -1>()
    };
    const A_MASK_1: M128I = unsafe {
        M128I::const_i8::<-1, -1, -1,  0, -1, -1, -1,  2, -1, -1, -1,  4, -1, -1, -1,  6>()
    };

    const R_MASK_2: M128I = unsafe {
        M128I::const_i8::< 8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14, -1, -1, -1>()
    };
    const G_MASK_2: M128I = unsafe {
        M128I::const_i8::<-1,  8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14, -1, -1>()
    };
    const B_MASK_2: M128I = unsafe {
        M128I::const_i8::<-1, -1,  8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14, -1>()
    };
    const A_MASK_2: M128I = unsafe {
        M128I::const_i8::<-1, -1, -1,  8, -1, -1, -1, 10, -1, -1, -1, 12, -1, -1, -1, 14>()
    };

    let transparent_vec = M128I::set1_epi16(transparent_color as i16);

    for _ in 0..pixel_blocks {
        // 8ピクセル取得
        let pixel = M128I::loadu_si128(data.cast::<M128I>()).be_epi16();

        // alpha作成
        let a_vec = pixel.cmpeq_epi16(transparent_vec).not_si128().srli_epi16::<8>();

        // マスクで色を分離
        let (mut r_vec, mut g_vec, mut b_vec) = get_rgb_vec(pixel);

        // 位置移動
        r_vec = r_vec.srli_epi16::<11>();
        g_vec = g_vec.srli_epi16::<5>();

        // 色戻し
        r_vec = r_vec.slli_epi16::<3>() | r_vec.srli_epi16::<2>();
        g_vec = g_vec.slli_epi16::<2>() | g_vec.srli_epi16::<4>();
        b_vec = b_vec.slli_epi16::<3>() | b_vec.srli_epi16::<2>();

        // rgbaに合成
        let rgb_1 = r_vec.shuffle_epi8(R_MASK_1) | g_vec.shuffle_epi8(G_MASK_1) | b_vec.shuffle_epi8(B_MASK_1) | a_vec.shuffle_epi8(A_MASK_1);
        let rgb_2 = r_vec.shuffle_epi8(R_MASK_2) | g_vec.shuffle_epi8(G_MASK_2) | b_vec.shuffle_epi8(B_MASK_2) | a_vec.shuffle_epi8(A_MASK_2);

        rgb_1.storeu_si128(buf.cast::<M128I>());
        buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());
        rgb_2.storeu_si128(buf.cast::<M128I>());

        data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
        buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());
    }
    
    scalar::decode_to_rgba8888_alpha_be(data, buf, transparent_color, remainder)
}

encode_from_endian!(
    "big",
    be_epi16,
    decode_to_rgb888_be,
    decode_to_rgb565_be,
    decode_to_rgba8888_be,
    decode_to_rgba8888_alpha_be
);

encode_from_endian!(
    "little",
    le_epi16,
    decode_to_rgb888_le,
    decode_to_rgb565_le,
    decode_to_rgba8888_le,
    decode_to_rgba8888_alpha_le
);

#[cfg(test)]
mod tests {
    use crate::decode::logic::scalar;
    use crate::pixel::PIXEL_BYTES;

    use crate::decode::logic::tests::{NUM_PIXELS, RGB565_DATA};

    #[test]
    fn encode_rgb565_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut simd_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        unsafe {
            scalar::decode_to_rgb565_be(RGB565_DATA.as_ptr().cast::<u8>(), scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::decode_to_rgb565_be(RGB565_DATA.as_ptr().cast::<u8>(), simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::decode_to_rgb565_le(RGB565_DATA.as_ptr().cast::<u8>(), scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::decode_to_rgb565_le(RGB565_DATA.as_ptr().cast::<u8>(), simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn decode_rgba8888_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut simd_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        unsafe {
            scalar::decode_to_rgba8888_be(RGB565_DATA.as_ptr().cast::<u8>(), scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::decode_to_rgba8888_be(RGB565_DATA.as_ptr().cast::<u8>(), simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::decode_to_rgba8888_le(RGB565_DATA.as_ptr().cast::<u8>(), scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::decode_to_rgba8888_le(RGB565_DATA.as_ptr().cast::<u8>(), simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn decode_rgba8888_alpha_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * 4];
        let mut simd_buf = [0; NUM_PIXELS * 4];

        let transparent_color = crate::rgb_to_pixel([255, 255, 255]);

        unsafe {
            scalar::decode_to_rgba8888_alpha_be(RGB565_DATA.as_ptr().cast::<u8>(), scalar_buf.as_mut_ptr(), transparent_color, NUM_PIXELS);
            super::decode_to_rgba8888_alpha_be(RGB565_DATA.as_ptr().cast::<u8>(), simd_buf.as_mut_ptr(), transparent_color, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::decode_to_rgba8888_alpha_le(RGB565_DATA.as_ptr().cast::<u8>(), scalar_buf.as_mut_ptr(), transparent_color, NUM_PIXELS);
            super::decode_to_rgba8888_alpha_le(RGB565_DATA.as_ptr().cast::<u8>(), simd_buf.as_mut_ptr(), transparent_color, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }
}