#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]

#[cfg(target_arch = "x86")]
use ::core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use ::core::arch::x86_64::*;

use ::core::mem::transmute;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct M128I(pub __m128i);

impl M128I {
    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn loadu_si128(mem_addr: *const M128I) -> M128I {
        M128I(_mm_loadu_si128(mem_addr.cast::<__m128i>()))
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn storeu_si128(self, mem_addr: *mut M128I) {
        _mm_storeu_si128(mem_addr.cast::<__m128i>(), self.0);
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn slli_epi16<const IMM8: i32>(self) -> M128I {
        M128I(_mm_slli_epi16::<IMM8>(self.0))
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn srli_epi16<const IMM8: i32>(self) -> M128I {
        M128I(_mm_srli_epi16::<IMM8>(self.0))
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn and_si128(self, a: M128I) -> M128I {
        M128I(_mm_and_si128(self.0, a.0))
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn or_si128(self, a: M128I) -> M128I {
        M128I(_mm_or_si128(self.0, a.0))
    }

    #[inline]
    #[target_feature(enable = "ssse3")]
    pub unsafe fn shuffle_epi8(self, a: M128I) -> M128I {
        M128I(_mm_shuffle_epi8(self.0, a.0))
    }
    
    // ---- 追加関数 ----

    #[inline(always)]
    pub const unsafe fn new_i8(e15: i8, e14: i8, e13: i8, e12: i8, e11: i8, e10: i8, e9: i8, e8: i8, e7: i8, e6: i8, e5: i8, e4: i8, e3: i8, e2: i8, e1: i8, e0: i8) -> M128I {
        M128I(transmute([e15, e14, e13, e12, e11, e10, e9, e8, e7, e6, e5, e4, e3, e2, e1, e0]))
    }

    #[inline]
    #[target_feature(enable = "ssse3")]
    pub unsafe fn swap_epi16(self) -> M128I {
        const MASK: M128I = unsafe { M128I::new_i8(1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14) };
        self.shuffle_epi8(MASK)
    }

    #[inline]
    #[target_feature(enable = "ssse3")]
    pub unsafe fn be_epi16(self) -> M128I {
        self.swap_epi16()
    }

    #[inline(always)]
    pub unsafe fn le_epi16(self) -> M128I {
        self
    }
}

impl ::core::ops::BitAnd for M128I {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { self.and_si128(rhs) }
    }
}

impl ::core::ops::BitOr for M128I {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { self.or_si128(rhs) }
    }
}
