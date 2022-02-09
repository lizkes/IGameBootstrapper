pub fn encrypt_message(message: &str) -> String {
    use aes::Aes256;
    use block_modes::{block_padding::Pkcs7, BlockMode, Cbc};

    type Aes256Cbc = Cbc<Aes256, Pkcs7>;

    let key: Vec<u8> = base64::decode("DP/B868Op9Ataw0l2YGtaS822jt26XWv7e3vMVa5zFI=").unwrap();
    let iv: Vec<u8> = base64::decode("3rqBYyUB02E5HLOCI2i/2A==").unwrap();
    let cipher = Aes256Cbc::new_from_slices(key.as_slice(), iv.as_slice()).unwrap();
    let encrypted_content = cipher.encrypt_vec(message.as_bytes());
    let content = base64::encode(encrypted_content);

    return content;
}
