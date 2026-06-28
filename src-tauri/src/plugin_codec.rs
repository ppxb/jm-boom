use aes::Aes256;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use ecb::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyInit};
use serde::de::DeserializeOwned;

const JM_SETTING_AES_SEEDS: [&str; 2] = ["185Hcomic3PAPP7R", "18comicAPPContent"];

pub(crate) fn decode_setting_payload<T>(body: &str, ts: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let value = serde_json::from_str::<serde_json::Value>(body)
        .unwrap_or_else(|_| serde_json::Value::String(body.to_string()));
    let decoded = decode_value(value, ts);

    serde_json::from_value(decoded).map_err(|error| format!("Invalid setting payload: {error}"))
}

fn decode_value(value: serde_json::Value, ts: &str) -> serde_json::Value {
    match value {
        serde_json::Value::String(raw) => {
            let raw = raw.trim();
            if raw.is_empty() {
                return serde_json::Value::String(String::new());
            }

            serde_json::from_str::<serde_json::Value>(raw)
                .map(|parsed| decode_value(parsed, ts))
                .unwrap_or_else(|_| serde_json::Value::String(raw.to_string()))
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(values),
        serde_json::Value::Object(object) => {
            if let Some(data) = object.get("data").and_then(|data| data.as_str()) {
                let raw_data = data.trim();
                if !raw_data.is_empty() {
                    if let Some(normalized) = normalize_base64(raw_data) {
                        if let Some(decrypted) = decrypt_data_field(&normalized, ts) {
                            return decrypted;
                        }
                    }

                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw_data) {
                        return parsed;
                    }
                }
            }

            serde_json::Value::Object(object)
        }
        value => value,
    }
}

fn decrypt_data_field(data: &str, ts: &str) -> Option<serde_json::Value> {
    let ts = ts.trim();
    if ts.is_empty() {
        return None;
    }

    for seed in JM_SETTING_AES_SEEDS {
        let key = md5_hex(&format!("{ts}{seed}"));
        let Ok(text) = decrypt_base64_with_key(data, &key) else {
            continue;
        };
        if text.trim().is_empty() {
            continue;
        }

        return serde_json::from_str::<serde_json::Value>(text.trim())
            .ok()
            .or_else(|| Some(serde_json::Value::String(text)));
    }

    None
}

fn normalize_base64(raw: &str) -> Option<String> {
    let compact = raw
        .trim()
        .replace(char::is_whitespace, "")
        .replace('-', "+")
        .replace('_', "/");
    if compact.is_empty() {
        return None;
    }

    let body = compact.trim_end_matches('=');
    if body.is_empty()
        || !body
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '+' | '/'))
    {
        return None;
    }

    match body.len() % 4 {
        1 => None,
        0 => Some(body.to_string()),
        remainder => Some(format!("{body}{}", "=".repeat(4 - remainder))),
    }
}

fn decrypt_base64_with_key(data: &str, key: &str) -> Result<String, String> {
    let encrypted = BASE64_STANDARD
        .decode(data)
        .map_err(|error| format!("Invalid encrypted data: {error}"))?;
    let decrypted = ecb::Decryptor::<Aes256>::new_from_slice(key.as_bytes())
        .map_err(|error| format!("Invalid AES key: {error}"))?
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted)
        .map_err(|error| format!("Failed to decrypt response: {error}"))?;

    String::from_utf8(decrypted).map_err(|error| format!("Invalid decrypted text: {error}"))
}

fn md5_hex(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}
