use super::{error::JmResult, API_SECRET};
use aes::Aes256;
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyInit, KeyIvInit};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

type Aes256EcbDec = ecb::Decryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;

/// Decrypt JM API response data
pub fn decrypt_data(data: &str, ts: &str) -> JmResult<String> {
    let key = md5_hex(&format!("{ts}{API_SECRET}"));
    decrypt_base64(&data, &key)
}

/// Decrypt base64-encoded data with AES-256-ECB
pub fn decrypt_base64(data: &str, key: &str) -> JmResult<String> {
    let encrypted = BASE64
        .decode(data)
        .map_err(|e| super::error::JmError::Decrypt(format!("Invalid base64: {e}")))?;

    let decrypted = Aes256EcbDec::new_from_slice(key.as_bytes())
        .map_err(|e| super::error::JmError::Decrypt(format!("Invalid AES key: {e}")))?
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted)
        .map_err(|e| super::error::JmError::Decrypt(format!("Decryption failed: {e}")))?;

    String::from_utf8(decrypted)
        .map_err(|e| super::error::JmError::Decrypt(format!("Invalid UTF-8: {e}")))
}

/// Decrypt with AES-256-ECB using provided key
pub fn decrypt_aes256_ecb(encrypted: &str, key: &str) -> JmResult<String> {
    decrypt_base64(encrypted, key)
}

/// Decrypt chapter manifest using AES-256-CBC
pub fn decrypt_aes256_cbc(encrypted: &str, ts: &str) -> JmResult<String> {
    let key_bytes = md5_hex(ts).into_bytes();
    let mut key = [0u8; 32];
    key[..key_bytes.len().min(32)].copy_from_slice(&key_bytes[..key_bytes.len().min(32)]);

    let iv = &key[..16];

    let encrypted_bytes = BASE64
        .decode(encrypted)
        .map_err(|e| super::error::JmError::Decrypt(format!("Base64 decode failed: {e}")))?;

    let decrypted = Aes256CbcDec::new(&key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted_bytes)
        .map_err(|e| super::error::JmError::Decrypt(format!("AES-CBC decrypt failed: {e}")))?;

    String::from_utf8(decrypted)
        .map_err(|e| super::error::JmError::Decrypt(format!("UTF-8 decode failed: {e}")))
}

fn md5_hex(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}
