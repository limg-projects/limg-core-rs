/// Limg 形式のASCII 4バイトシグネチャ
pub const IMAGE_SIGNATURE: [u8; 4] = *b"LIMG";

/// 比較時に使用するu32形式シグネチャ
pub const IMAGE_SIGNATURE_U32_NE: u32 = u32::from_ne_bytes(IMAGE_SIGNATURE);

/// Limg形式のヘッダーサイズ
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{ImageHeader, IMAGE_HEADER_SIZE};
/// 
/// assert_eq!(IMAGE_HEADER_SIZE, size_of::<ImageHeader>());
/// ```
pub const IMAGE_HEADER_SIZE: usize = 12;

/// Limg形式の現行バージョン
pub const IMAGE_CURRENT_VARSION: u8 = 1;

/// データ部エンディアン用ビットマスク
///
/// フラグが立っているならビッグエンディアン、そうでないならリトルエンディアン
pub const IMAGE_FLAG_ENDIAN_BIT: u8 = 0b00000001;

/// データ部透明色使用ビットマスク
/// 
/// フラグが立っているなら透明色使用、そうでないなら使用しない
pub const IMAGE_FLAG_USE_TRANSPARENT_BIT: u8 = 0b00000010;

/// バイナリに直接変換できるヘッダー形式
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImageHeader {
    /// u32形式のシグネチャ
    pub signature: u32,
    /// 形式バージョン
    pub version: u8,
    /// フォーマットフラグ.
    pub flag: u8,
    /// 画像の横幅
    pub width: u16,
    /// 画像の縦幅
    pub height: u16,
    /// RGB565形式の透明色
    pub transparent_color: u16,
}
