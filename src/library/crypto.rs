use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};

pub fn encrypt_message(message: &str) -> String {
    type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

    let key: Vec<u8> = base64::decode("DP/B868Op9Ataw0l2YGtaS822jt26XWv7e3vMVa5zFI=").unwrap();
    let iv: Vec<u8> = base64::decode("3rqBYyUB02E5HLOCI2i/2A==").unwrap();
    let encryptor = Aes256CbcEnc::new_from_slices(key.as_slice(), iv.as_slice()).unwrap();
    let ciphertext = encryptor.encrypt_padded_vec_mut::<Pkcs7>(message.as_bytes());
    let content = base64::encode(ciphertext);

    return content;
}
