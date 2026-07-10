use super::{error::JmResult, API_SECRET};
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyInit};
use aes::Aes256;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

type Aes256EcbDec = ecb::Decryptor<Aes256>;

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

fn md5_hex(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}
