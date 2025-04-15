#![allow(unsafe_op_in_unsafe_fn)]

use crate::pixel::{PIXEL_BYTES, PIXEL_R_MASK, PIXEL_G_MASK, PIXEL_B_MASK};
use crate::decode::logic::{scalar, decode_logic_fn};
use crate::common::color::ColorType;
use crate::common::logic::x86_64::M256I;

const PIXEL_BLOCK_LEN: usize = 16; // u16(16 bit) * 16 = 256 bit

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn get_rgb_vec(pixel: M256I) -> (M256I, M256I, M256I) {
    const R_MASK: M256I = unsafe { M256I::const1_u16::<PIXEL_R_MASK>() };
    const G_MASK: M256I = unsafe { M256I::const1_u16::<PIXEL_G_MASK>() };
    const B_MASK: M256I = unsafe { M256I::const1_u16::<PIXEL_B_MASK>() };

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
#[target_feature(enable = "avx2")]
pub unsafe fn decode_from_rgb565_swap(mut data: *const u8, mut buf: *mut u8, num_pixels: usize) {
    let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
    let remainder = num_pixels % PIXEL_BLOCK_LEN;

    for _ in 0..pixel_blocks {
        let pixel = M256I::loadu_si256(data.cast::<M256I>()).swap_epi16();

        pixel.storeu_si256(buf.cast::<M256I>());
        
        data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
        buf = buf.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
    }

    scalar::decode_from_rgb565_swap(data, buf, remainder)
}

macro_rules! decode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident, $rgba8888_alpha: ident) => {

        // -- rgb888 ------------------------------

        #[inline]
        #[target_feature(enable = "avx2")]
        pub unsafe fn $rgb888(mut data: *const u8, mut buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;

            const RGB_MASK: M256I = unsafe { M256I::const_i8::<
                0, -1, -1,  2, -1, -1,  4, -1, -1,  6, -1, -1, -1, -1, -1, -1,
                0, -1, -1,  2, -1, -1,  4, -1, -1,  6, -1, -1, -1, -1, -1, -1,
            >() };

            const BEFORE_MASK : M256I = unsafe { M256I::const_i32::<6, -1, -1, -1, -1, -1, -1, -1>() };

            // バッファオーバーしないための前後ピクセルを加味する
            if num_pixels < PIXEL_BLOCK_LEN + 2 + 2 {
                scalar::$rgb888(data, buf, num_pixels);
                return;
            }

            // 先頭の2ピクセル先に処理する
            scalar::$rgb888(data, buf, 2);
            let num_pixels = num_pixels - 2;

            // 2ピクセル部分進めておく
            data = data.add(PIXEL_BYTES * 2);
            // 後半処理のため2バイトずらす
            buf = buf.add(2);            

            let mut before_vec = M256I::loadu_si256(buf.cast::<M256I>()).blend_epi32::<0b11111110>(M256I::setzero_si256());
        
            let pixel_blocks = (num_pixels - 2) / PIXEL_BLOCK_LEN;
            let remainder = num_pixels - (PIXEL_BLOCK_LEN * pixel_blocks);

            for _ in 0..pixel_blocks {
                // 8ピクセル取得
                let pixel = M256I::loadu_si256(data.cast::<M256I>()).$endian_fn();
        
                // マスクで色を分離
                let (r_vec, g_vec, b_vec) = get_rgb_vec(pixel);

                let rgb_1 = r_vec.shuffle_epi8(RGB_MASK).slli_si256::<4>() |
                    g_vec.shuffle_epi8(RGB_MASK).slli_si256::<5>() |
                    b_vec.shuffle_epi8(RGB_MASK).slli_si256::<6>();

                
                let rgb_2 = r_vec.srli_si256::<8>().shuffle_epi8(RGB_MASK) |
                g_vec.srli_si256::<8>().shuffle_epi8(RGB_MASK).slli_si256::<1>() |
                b_vec.srli_si256::<8>().shuffle_epi8(RGB_MASK).slli_si256::<2>();

                let mut rgb = rgb_1.permute2x128_si256::<0x20>(rgb_2) | before_vec;
                before_vec = rgb.permutevar8x32_epi32(BEFORE_MASK);
                
                // 8ピクセル書き込み
                rgb.storeu_si256(buf.cast::<M256I>());
                buf = buf.add(8 * COLOR_TYPE.bytes_per_pixel());

                rgb = rgb_1.permute2x128_si256::<0x31>(rgb_2) | before_vec;
                before_vec = rgb.permutevar8x32_epi32(BEFORE_MASK);

                // 8ピクセル書き込み
                rgb.storeu_si256(buf.cast::<M256I>());
        
                data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                buf = buf.add(8 * COLOR_TYPE.bytes_per_pixel());
            }
            
            scalar::$rgb888(data, buf.add(4), remainder)
        }

        // -- rgb565 ------------------------------

        #[inline(always)]
        #[cfg(target_endian = $endian)]
        pub unsafe fn $rgb565(data: *const u8, buf: *mut u8, num_pixels: usize) {
            scalar::decode_from_rgb565_direct(data, buf, num_pixels)
        }
        
        #[inline]
        #[target_feature(enable = "avx2")]
        #[cfg(not(target_endian = $endian))]
        pub unsafe fn $rgb565(data: *const u8, buf: *mut u8, num_pixels: usize) {
            decode_from_rgb565_swap(data, buf, num_pixels)
        }

        // // -- rgba8888 ----------------------------

        #[inline]
        #[target_feature(enable = "avx2")]
        pub unsafe fn $rgba8888(mut data: *const u8, mut buf: *mut u8, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;

            const RGBA_MASK: M256I = unsafe { M256I::const_i8::<
                0, -1, -1, -1, 2, -1, -1, -1, 4, -1, -1, -1, 6, -1, -1, -1,
                0, -1, -1, -1, 2, -1, -1, -1, 4, -1, -1, -1, 6, -1, -1, -1,
            >() };
        
            const ALPHA_VEC: M256I = unsafe { M256I::const_u8::<
                0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
                0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
            >() };

            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;

            for _ in 0..pixel_blocks {
                // 16ピクセル取得
                let pixel = M256I::loadu_si256(data.cast::<M256I>()).$endian_fn();
        
                // マスクで色を分離
                let (r_vec, g_vec, b_vec) = get_rgb_vec(pixel);

                let rgb_1 = r_vec.shuffle_epi8(RGBA_MASK) |
                    g_vec.shuffle_epi8(RGBA_MASK).slli_si256::<1>() |
                    b_vec.shuffle_epi8(RGBA_MASK).slli_si256::<2>() |
                    ALPHA_VEC;

                let rgb_2 = r_vec.srli_si256::<8>().shuffle_epi8(RGBA_MASK) |
                    g_vec.srli_si256::<8>().shuffle_epi8(RGBA_MASK).slli_si256::<1>() |
                    b_vec.srli_si256::<8>().shuffle_epi8(RGBA_MASK).slli_si256::<2>() |
                    ALPHA_VEC;
                
                let mut rgb = rgb_1.permute2x128_si256::<0x20>(rgb_2);

                // 8ピクセル書き込み
                rgb.storeu_si256(buf.cast::<M256I>());
                buf = buf.add(8 * COLOR_TYPE.bytes_per_pixel());

                rgb = rgb_1.permute2x128_si256::<0x31>(rgb_2);

                // 8ピクセル書き込み
                rgb.storeu_si256(buf.cast::<M256I>());
        
                data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                buf = buf.add(8 * COLOR_TYPE.bytes_per_pixel());
            }
            
            scalar::$rgba8888(data, buf, remainder)
        }

        // -- rgba8888 alpha ----------------------

        #[inline]
        #[target_feature(enable = "avx2")]
        pub unsafe fn $rgba8888_alpha(mut data: *const u8, mut buf: *mut u8, transparent_color: u16, num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;

            const RGBA_MASK: M256I = unsafe { M256I::const_i8::<
                0, -1, -1, -1, 2, -1, -1, -1, 4, -1, -1, -1, 6, -1, -1, -1,
                0, -1, -1, -1, 2, -1, -1, -1, 4, -1, -1, -1, 6, -1, -1, -1,
            >() };
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            let transparent_vec = M256I::set1_epi16(transparent_color as i16);
        
            for _ in 0..pixel_blocks {
                // 16ピクセル取得
                let pixel = M256I::loadu_si256(data.cast::<M256I>()).$endian_fn();
        
                // alpha作成
                let a_vec = pixel.cmpeq_epi16(transparent_vec).not_si256().srli_epi16::<8>();
        
                // マスクで色を分離
                let (r_vec, g_vec, b_vec) = get_rgb_vec(pixel);

                let rgb_1 = r_vec.shuffle_epi8(RGBA_MASK) |
                    g_vec.shuffle_epi8(RGBA_MASK).slli_si256::<1>() |
                    b_vec.shuffle_epi8(RGBA_MASK).slli_si256::<2>() |
                    a_vec.shuffle_epi8(RGBA_MASK).slli_si256::<3>();

                let rgb_2 = r_vec.srli_si256::<8>().shuffle_epi8(RGBA_MASK) |
                    g_vec.srli_si256::<8>().shuffle_epi8(RGBA_MASK).slli_si256::<1>() |
                    b_vec.srli_si256::<8>().shuffle_epi8(RGBA_MASK).slli_si256::<2>() |
                    a_vec.srli_si256::<8>().shuffle_epi8(RGBA_MASK).slli_si256::<3>();
                
                let mut rgb = rgb_1.permute2x128_si256::<0x20>(rgb_2);

                // 8ピクセル書き込み
                rgb.storeu_si256(buf.cast::<M256I>());
                buf = buf.add(8 * COLOR_TYPE.bytes_per_pixel());

                rgb = rgb_1.permute2x128_si256::<0x31>(rgb_2);

                // 8ピクセル書き込み
                rgb.storeu_si256(buf.cast::<M256I>());
        
                data = data.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                buf = buf.add(8 * COLOR_TYPE.bytes_per_pixel());
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
    use crate::common::color::ColorType;
    use crate::spec::{ImageSpec, DataEndian};

    use crate::decode::logic::tests::{NUM_PIXELS, RGB565_DATA_BE, RGB565_DATA_LE};

    #[test]
    fn decode_logic_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
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
    fn decode_rgb888_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
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
    fn decode_rgb565_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
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
    fn decode_rgba8888_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
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
    fn decode_rgba8888_alpha_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
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