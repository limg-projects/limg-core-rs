#![allow(unsafe_op_in_unsafe_fn)]

use crate::decode_logic_fn;
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

    // 抽出 & 移動
    let r_vec = (pixel & R_MASK).srli_epi16::<11>();
    let g_vec = (pixel & G_MASK).srli_epi16::<5>();
    let b_vec =  pixel & B_MASK;

    // rgb565 -> rgb888
    (
        (r_vec.slli_epi16::<3>() | r_vec.srli_epi16::<2>()),
        (g_vec.slli_epi16::<2>() | g_vec.srli_epi16::<4>()),
        (b_vec.slli_epi16::<3>() | b_vec.srli_epi16::<2>())
    )
}

#[inline]
#[target_feature(enable = "ssse3")]
pub unsafe fn decode_from_rgb565_swap(mut data: *const u8, mut buf: *mut u8, num_pixels: usize) {
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

macro_rules! decode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident, $rgba8888_alpha: ident) => {
        // -- rgb888 ------------------------------

        #[inline]
        #[target_feature(enable = "ssse3")]
        pub unsafe fn $rgb888(mut data: *const u8, mut buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;

            const RGB_MASK: M128I = unsafe {
                M128I::const_i8::<0, -1, -1, 2, -1, -1, 4, -1, -1, 6, -1, -1, -1, -1, -1, -1>()
            };
        
            // バッファオーバーしないための後ピクセルを加味する
            if num_pixels < PIXEL_BLOCK_LEN + 2 {
                return scalar::$rgb888(data, buf, num_pixels);
            }
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            for _ in 0..pixel_blocks {
                // 8ピクセル読み込み
                let pixel = M128I::loadu_si128(data.cast::<M128I>()).$endian_fn();
        
                // マスクで色を分離
                let (r_vec, g_vec, b_vec) = get_rgb_vec(pixel);
        
                // rgbに合成（前半4ピクセル）
                let mut rgb = r_vec.shuffle_epi8(RGB_MASK) |
                    g_vec.shuffle_epi8(RGB_MASK).slli_si128::<1>() |
                    b_vec.shuffle_epi8(RGB_MASK).slli_si128::<2>();
        
                // 4ピクセル書き込み
                rgb.storeu_si128(buf.cast::<M128I>());
                buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());
        
                // rgbに合成（後半4ピクセル）
                rgb = r_vec.srli_si128::<8>().shuffle_epi8(RGB_MASK) |
                    g_vec.srli_si128::<8>().shuffle_epi8(RGB_MASK).slli_si128::<1>() |
                    b_vec.srli_si128::<8>().shuffle_epi8(RGB_MASK).slli_si128::<2>();
        
                // 4ピクセル書き込み
                rgb.storeu_si128(buf.cast::<M128I>());
        
                data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());
            }
            
            scalar::$rgb888(data, buf, remainder)
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
        pub unsafe fn $rgba8888(mut data: *const u8, mut buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;

            const RGB_MASK: M128I = unsafe {
                M128I::const_i8::<0, -1, -1, -1, 2, -1, -1, -1, 4, -1, -1, -1, 6, -1, -1, -1>()
            };
        
            const ALPHA_VEC: M128I = unsafe {
                M128I::const_u8::<0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255>()
            };

            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;

            for _ in 0..pixel_blocks {
                // 8ピクセル取得
                let pixel = M128I::loadu_si128(data.cast::<M128I>()).$endian_fn();

                // マスクで色を分離
                let (r_vec, g_vec, b_vec) = get_rgb_vec(pixel);

                // rgbに合成（前半4ピクセル）
                let mut rgb = r_vec.shuffle_epi8(RGB_MASK) |
                    g_vec.shuffle_epi8(RGB_MASK).slli_si128::<1>() |
                    b_vec.shuffle_epi8(RGB_MASK).slli_si128::<2>() |
                    ALPHA_VEC;
                
                // 4ピクセル書き込み
                rgb.storeu_si128(buf.cast::<M128I>());
                buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());    

                // rgbに合成（後半4ピクセル）
                rgb = r_vec.srli_si128::<8>().shuffle_epi8(RGB_MASK) |
                    g_vec.srli_si128::<8>().shuffle_epi8(RGB_MASK).slli_si128::<1>() |
                    b_vec.srli_si128::<8>().shuffle_epi8(RGB_MASK).slli_si128::<2>() |
                    ALPHA_VEC;

                // 4ピクセル書き込み
                rgb.storeu_si128(buf.cast::<M128I>());
        
                data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());
            }
            
            scalar::$rgba8888(data, buf, remainder)
        }

        // -- rgba8888 alpha ----------------------

        #[inline]
        #[target_feature(enable = "ssse3")]
        pub unsafe fn $rgba8888_alpha(mut data: *const u8, mut buf: *mut u8, transparent_color: u16, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;

            const RGBA_MASK: M128I = unsafe {
                M128I::const_i8::<0, -1, -1, -1, 2, -1, -1, -1, 4, -1, -1, -1, 6, -1, -1, -1>()
            };
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            let transparent_vec = M128I::set1_epi16(transparent_color as i16);
        
            for _ in 0..pixel_blocks {
                // 8ピクセル取得
                let pixel = M128I::loadu_si128(data.cast::<M128I>()).$endian_fn();
        
                // alpha作成
                let a_vec = pixel.cmpeq_epi16(transparent_vec).not_si128().srli_epi16::<8>();
        
                // マスクで色を分離
                let (r_vec, g_vec, b_vec) = get_rgb_vec(pixel);

                // rgbに合成（前半4ピクセル）
                let mut rgb = r_vec.shuffle_epi8(RGBA_MASK) |
                    g_vec.shuffle_epi8(RGBA_MASK).slli_si128::<1>() |
                    b_vec.shuffle_epi8(RGBA_MASK).slli_si128::<2>() |
                    a_vec.shuffle_epi8(RGBA_MASK).slli_si128::<3>();

                // 4ピクセル書き込み
                rgb.storeu_si128(buf.cast::<M128I>());
                buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());    

                // rgbに合成（後半4ピクセル）
                rgb = r_vec.srli_si128::<8>().shuffle_epi8(RGBA_MASK) |
                    g_vec.srli_si128::<8>().shuffle_epi8(RGBA_MASK).slli_si128::<1>() |
                    b_vec.srli_si128::<8>().shuffle_epi8(RGBA_MASK).slli_si128::<2>() |
                    a_vec.srli_si128::<8>().shuffle_epi8(RGBA_MASK).slli_si128::<3>();

                // 4ピクセル書き込み
                rgb.storeu_si128(buf.cast::<M128I>());

                data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                buf = buf.add(4 * COLOR_TYPE.bytes_per_pixel());
            }
            
            scalar::$rgba8888_alpha(data, buf, transparent_color, remainder)
        }
    };
}

decode_logic_fn!();

decode_from_endian!(
    "big",
    be_epi16,
    decode_to_rgb888_be,
    decode_to_rgb565_be,
    decode_to_rgba8888_be,
    decode_to_rgba8888_alpha_be
);

decode_from_endian!(
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
    use crate::pixel::ColorType;
    use crate::spec::{ImageSpec, DataEndian};

    use crate::decode::logic::tests::{NUM_PIXELS, RGB565_DATA_BE, RGB565_DATA_LE};

    #[test]
    fn decode_logic_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut a_buf = [0; NUM_PIXELS * ColorType::Rgba8888.bytes_per_pixel()];
        let mut b_buf = [0; NUM_PIXELS * ColorType::Rgba8888.bytes_per_pixel()];

        let rgb565_be_ptr = RGB565_DATA_BE.as_ptr().cast::<u8>();
        let rgb565_le_ptr = RGB565_DATA_LE.as_ptr().cast::<u8>();

        let mut spec = ImageSpec {
            width: NUM_PIXELS as u16,
            height: 1,
            transparent_color: Some(0xFF),
            data_endian: DataEndian::Big
        };

        unsafe {
            super::decode_logic(rgb565_be_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgb888);
            super::decode_to_rgb888_be(rgb565_be_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            super::decode_logic(rgb565_be_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgb565);
            super::decode_to_rgb565_be(rgb565_be_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            super::decode_logic(rgb565_be_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgba8888);
            super::decode_to_rgba8888_be(rgb565_be_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            spec.data_endian = DataEndian::Little;
            
            super::decode_logic(rgb565_le_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgb888);
            super::decode_to_rgb888_le(rgb565_le_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            super::decode_logic(rgb565_le_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgb565);
            super::decode_to_rgb565_le(rgb565_le_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            super::decode_logic(rgb565_le_ptr, a_buf.as_mut_ptr(), &spec, ColorType::Rgba8888);
            super::decode_to_rgba8888_le(rgb565_le_ptr, b_buf.as_mut_ptr(), NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
        }
    }

    #[test]
    fn decode_rgb888_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }
        
        let mut scalar_buf = [0; NUM_PIXELS * ColorType::Rgb888.bytes_per_pixel()];
        let mut simd_buf = [0; NUM_PIXELS * ColorType::Rgb888.bytes_per_pixel()];

        let rgb565_be_ptr = RGB565_DATA_BE.as_ptr().cast::<u8>();
        let rgb565_le_ptr = RGB565_DATA_LE.as_ptr().cast::<u8>();

        unsafe {
            scalar::decode_to_rgb888_be(rgb565_be_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::decode_to_rgb888_be(rgb565_be_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::decode_to_rgb888_le(rgb565_le_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::decode_to_rgb888_le(rgb565_le_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn decode_rgb565_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * ColorType::Rgb565.bytes_per_pixel()];
        let mut simd_buf = [0; NUM_PIXELS * ColorType::Rgb565.bytes_per_pixel()];

        let rgb565_be_ptr = RGB565_DATA_BE.as_ptr().cast::<u8>();
        let rgb565_le_ptr = RGB565_DATA_LE.as_ptr().cast::<u8>();

        unsafe {
            scalar::decode_to_rgb565_be(rgb565_be_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::decode_to_rgb565_be(rgb565_be_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::decode_to_rgb565_le(rgb565_le_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::decode_to_rgb565_le(rgb565_le_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn decode_rgba8888_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * ColorType::Rgba8888.bytes_per_pixel()];
        let mut simd_buf = [0; NUM_PIXELS * ColorType::Rgba8888.bytes_per_pixel()];

        let rgb565_be_ptr = RGB565_DATA_BE.as_ptr().cast::<u8>();
        let rgb565_le_ptr = RGB565_DATA_LE.as_ptr().cast::<u8>();

        unsafe {
            scalar::decode_to_rgba8888_be(rgb565_be_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::decode_to_rgba8888_be(rgb565_be_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::decode_to_rgba8888_le(rgb565_le_ptr, scalar_buf.as_mut_ptr(), NUM_PIXELS);
            super::decode_to_rgba8888_le(rgb565_le_ptr, simd_buf.as_mut_ptr(), NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }

    #[test]
    fn decode_rgba8888_alpha_x86_64_ssse3() {
        if !is_x86_feature_detected!("ssse3") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * ColorType::Rgba8888.bytes_per_pixel()];
        let mut simd_buf = [0; NUM_PIXELS * ColorType::Rgba8888.bytes_per_pixel()];

        let rgb565_be_ptr = RGB565_DATA_BE.as_ptr().cast::<u8>();
        let rgb565_le_ptr = RGB565_DATA_LE.as_ptr().cast::<u8>();

        let transparent_color = crate::pixel::rgb_to_pixel([255, 255, 255]);

        unsafe {
            scalar::decode_to_rgba8888_alpha_be(rgb565_be_ptr, scalar_buf.as_mut_ptr(), transparent_color, NUM_PIXELS);
            super::decode_to_rgba8888_alpha_be(rgb565_be_ptr, simd_buf.as_mut_ptr(), transparent_color, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);

        unsafe {
            scalar::decode_to_rgba8888_alpha_le(rgb565_le_ptr, scalar_buf.as_mut_ptr(), transparent_color, NUM_PIXELS);
            super::decode_to_rgba8888_alpha_le(rgb565_le_ptr, simd_buf.as_mut_ptr(), transparent_color, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, simd_buf);
    }
}