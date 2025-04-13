use limg_core::{decode::{decode, decode_data, decode_header}, encode::{encode, encode_data, encode_header, encoded_size}, pixel::ColorType};

fn encode_decode_test(color_type: ColorType) {
    let mut encode_buf = vec![0u8; 512 * 512 * 4];
    let mut decode_buf = vec![0u8; 512 * 512 * 4];

    let dir = std::fs::read_dir("tests/limg").unwrap();

    for item in dir.into_iter() {
        encode_buf.fill(0);
        decode_buf.fill(0);

        let path = item.unwrap().path();

        let data = std::fs::read(path).unwrap();

        let (spec, decoded_size) = decode(&data, &mut decode_buf, color_type).unwrap();

        let encoded_size = encode(&decode_buf[..decoded_size], &mut encode_buf, &spec, color_type).unwrap();

        assert_eq!(encode_buf[..encoded_size], data);
    }
}

fn encode_decode_header_data_test(color_type: ColorType) {
    let dir = std::fs::read_dir("tests/limg").unwrap();

    for item in dir.into_iter() {
        let path = item.unwrap().path();

        let data = std::fs::read(path).unwrap();

        let (spec, header_read_size) = decode_header(&data).unwrap();
        let mut decode_buf = vec![0u8; color_type.bytes_per_pixel() * spec.num_pixels()];

        let decoded_size = decode_data(&data[header_read_size..], &mut decode_buf, &spec, color_type).unwrap();

        let mut encode_buf = vec![0u8; encoded_size(&spec)];
        let mut encoded_size = 0;

        let written_header_size = encode_header(&mut encode_buf, &spec).unwrap();
        encoded_size += written_header_size;
        encoded_size += encode_data(&decode_buf[..decoded_size], &mut encode_buf[written_header_size..], &spec, color_type).unwrap();

        assert_eq!(encode_buf[..encoded_size], data);
    }
}

#[test]
fn limg_rgb888_test() {
    encode_decode_test(ColorType::Rgb888);
}

#[test]
fn limg_rgb565_test() {
    encode_decode_test(ColorType::Rgb565);
}

#[test]
fn limg_rgba8888_test() {
    encode_decode_test(ColorType::Rgba8888);
}

#[test]
fn limg_rgb888_header_data_test() {
    encode_decode_header_data_test(ColorType::Rgb888);
}

#[test]
fn limg_rgb565_header_data_test() {
    encode_decode_header_data_test(ColorType::Rgb565);
}

#[test]
fn limg_rgba8888_header_data_test() {
    encode_decode_header_data_test(ColorType::Rgba8888);
}