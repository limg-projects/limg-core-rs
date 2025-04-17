use ::core::fmt;

pub type Result<T> = ::core::result::Result<T, Error>;

/// エンコードおよびデコード時に発生する可能性があるエラー
#[derive(Debug)]
pub enum Error {
    /// 画像の幅および高さが0です。
    /// 
    /// エンコード時にサイズが0になる設定はできません。
    ZeroImageDimensions,

    /// 入力バッファの長さが足りません。
    InputBufferTooSmall,

    /// 出力バッファの長さが足りません。
    OutputBufferTooSmall,

    /// 画像形式がサポートされていません。
    /// 
    /// デコード時に発生する可能性があります。
    UnsupportedFormat,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ZeroImageDimensions => write!(f, "Image width or height is zero"),
            Error::InputBufferTooSmall => write!(f, "Input buffer is too small"),
            Error::OutputBufferTooSmall => write!(f, "Output buffer is too small"),
            Error::UnsupportedFormat => write!(f, "Unsupported image format or header"),
        }
    }
}

impl ::core::error::Error for Error {}