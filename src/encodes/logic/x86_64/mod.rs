

#[cfg(any(test, target_feature = "avx2"))]
mod avx2;
#[cfg(target_feature = "avx2")]
pub use avx2::encode_logic;

#[cfg(any(test, all(not(target_feature = "avx2"), target_feature = "ssse3")))]
mod ssse3;
#[cfg(all(not(target_feature = "avx2"), target_feature = "ssse3"))]
pub use ssse3::encode_logic;

#[cfg(all(not(target_feature = "avx2"), not(target_feature = "ssse3")))]
pub use crate::encodes::logic::scalar::encode_logic;
