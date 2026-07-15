use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasmer::FunctionEnvMut;

use super::HostState;
use crate::source::runtime::store::DescriptorValue;

#[derive(Clone, Debug)]
enum DefaultValue {
    Data(Vec<u8>),
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    StringArray(Vec<String>),
    Null,
}

#[derive(Default)]
pub(super) struct UserDefaults {
    values: HashMap<String, DefaultValue>,
}

impl UserDefaults {
    pub(super) fn new() -> Self {
        Self::default()
    }

    fn get(&self, key: &str) -> Option<&DefaultValue> {
        self.values.get(key)
    }

    fn set(&mut self, key: String, value: DefaultValue) {
        self.values.insert(key, value);
    }
}

const INVALID_KEY: i32 = -1;
const INVALID_VALUE: i32 = -2;
const FAILED_ENCODING: i32 = -3;
const FAILED_DECODING: i32 = -4;

pub(super) fn get(mut env: FunctionEnvMut<HostState>, pointer: i32, length: i32) -> i32 {
    let Ok(key) = env.data().read_string(&env, pointer, length) else {
        return INVALID_KEY;
    };
    let Some(value) = env.data().defaults.get(&key).cloned() else {
        return INVALID_VALUE;
    };
    let bytes = match value {
        DefaultValue::Data(data) => data,
        DefaultValue::Bool(value) => encode(&value),
        DefaultValue::Int(value) => encode(&value),
        DefaultValue::Float(value) => encode(&value),
        DefaultValue::String(value) => encode(&value),
        DefaultValue::StringArray(value) => encode(&value),
        DefaultValue::Null => return INVALID_VALUE,
    };
    if bytes.is_empty() {
        return FAILED_ENCODING;
    }
    env.data_mut()
        .descriptors
        .insert(DescriptorValue::Encoded(bytes))
}

pub(super) fn set(
    mut env: FunctionEnvMut<HostState>,
    key_pointer: i32,
    key_length: i32,
    kind: i32,
    value_pointer: i32,
) -> i32 {
    let Ok(key) = env.data().read_string(&env, key_pointer, key_length) else {
        return INVALID_KEY;
    };
    let Ok(bytes) = env.data().read_item_bytes(&env, value_pointer) else {
        return FAILED_DECODING;
    };
    let value = match kind {
        0 => Ok(DefaultValue::Data(bytes)),
        1 => decode(&bytes).map(DefaultValue::Bool),
        2 => decode(&bytes).map(DefaultValue::Int),
        3 => decode(&bytes).map(DefaultValue::Float),
        4 => decode(&bytes).map(DefaultValue::String),
        5 => decode(&bytes).map(DefaultValue::StringArray),
        6 => Ok(DefaultValue::Null),
        _ => return INVALID_VALUE,
    };
    let Ok(value) = value else {
        return FAILED_DECODING;
    };
    env.data_mut().defaults.set(key, value);
    0
}

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    postcard::to_allocvec(value).unwrap_or_default()
}

fn decode<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(bytes)
}
