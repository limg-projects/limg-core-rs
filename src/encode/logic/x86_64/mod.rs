

#[cfg(any(test, target_feature = "avx2"))]
mod avx2;
#[cfg(target_feature = "avx2")]
pub use avx2::*;

#[cfg(any(test, all(not(target_feature = "avx2"), target_feature = "sse4.1")))]
mod ssse3;
#[cfg(all(not(target_feature = "avx2"), target_feature = "sse4.1"))]
pub use ssse3::*;

#[cfg(all(not(target_feature = "avx2"), not(target_feature = "sse4.1")))]
pub use crate::encode::logic::scalar::*;
