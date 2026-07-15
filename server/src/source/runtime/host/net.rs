use std::{io::Read, time::Duration};

use image::ImageReader;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, Url,
};
use wasmer::FunctionEnvMut;

use super::{canvas::ImageData, html, HostState};
use crate::source::runtime::store::DescriptorValue;

const MAX_RESPONSE_BYTES: usize = 64 * 1024 * 1024;
const SUCCESS: i32 = 0;
const INVALID_DESCRIPTOR: i32 = -1;
const INVALID_STRING: i32 = -2;
const INVALID_METHOD: i32 = -3;
const INVALID_URL: i32 = -4;
const INVALID_HTML: i32 = -5;
const MISSING_DATA: i32 = -7;
const MISSING_RESPONSE: i32 = -8;
const REQUEST_ERROR: i32 = -10;
const FAILED_MEMORY_WRITE: i32 = -11;
const NOT_AN_IMAGE: i32 = -12;

#[derive(Clone, Copy)]
pub(super) enum HttpMethod {
    Get,
    Post,
    Put,
    Head,
    Delete,
    Patch,
    Options,
    Connect,
    Trace,
}

#[derive(Clone)]
pub(crate) struct NetRequest {
    pub(super) method: HttpMethod,
    pub(super) url: Option<Url>,
    pub(super) headers: HeaderMap,
    pub(super) body: Option<Vec<u8>>,
    pub(super) timeout: Option<Duration>,
    pub(super) response: Option<NetResponse>,
}

#[derive(Clone)]
pub(super) struct NetResponse {
    pub(super) url: Url,
    pub(super) status: u16,
    pub(super) headers: HeaderMap,
    pub(super) data: Vec<u8>,
}

impl HttpMethod {
    fn from_i32(value: i32) -> Option<Self> {
        Some(match value {
            0 => Self::Get,
            1 => Self::Post,
            2 => Self::Put,
            3 => Self::Head,
            4 => Self::Delete,
            5 => Self::Patch,
            6 => Self::Options,
            7 => Self::Connect,
            8 => Self::Trace,
            _ => return None,
        })
    }

    fn as_reqwest(self) -> Method {
        match self {
            Self::Get => Method::GET,
            Self::Post => Method::POST,
            Self::Put => Method::PUT,
            Self::Head => Method::HEAD,
            Self::Delete => Method::DELETE,
            Self::Patch => Method::PATCH,
            Self::Options => Method::OPTIONS,
            Self::Connect => Method::CONNECT,
            Self::Trace => Method::TRACE,
        }
    }
}

impl NetRequest {
    fn new(method: HttpMethod) -> Self {
        Self {
            method,
            url: None,
            headers: HeaderMap::new(),
            body: None,
            timeout: None,
            response: None,
        }
    }
}

pub(super) fn init(mut env: FunctionEnvMut<HostState>, method: i32) -> i32 {
    let Some(method) = HttpMethod::from_i32(method) else {
        return INVALID_METHOD;
    };
    env.data_mut()
        .descriptors
        .insert(DescriptorValue::Request(Box::new(NetRequest::new(method))))
}

pub(super) fn set_url(
    mut env: FunctionEnvMut<HostState>,
    descriptor: i32,
    pointer: i32,
    length: i32,
) -> i32 {
    let Ok(value) = env.data().read_string(&env, pointer, length) else {
        return INVALID_STRING;
    };
    let Ok(url) = Url::parse(&value) else {
        return INVALID_URL;
    };
    let Some(request) = env
        .data_mut()
        .descriptors
        .get_mut(descriptor)
        .and_then(DescriptorValue::as_request_mut)
    else {
        return INVALID_DESCRIPTOR;
    };
    request.url = Some(url);
    SUCCESS
}

pub(super) fn set_header(
    mut env: FunctionEnvMut<HostState>,
    descriptor: i32,
    key_pointer: i32,
    key_length: i32,
    value_pointer: i32,
    value_length: i32,
) -> i32 {
    let Ok(key) = env.data().read_string(&env, key_pointer, key_length) else {
        return INVALID_STRING;
    };
    let Ok(value) = env.data().read_string(&env, value_pointer, value_length) else {
        return INVALID_STRING;
    };
    let Ok(key) = HeaderName::try_from(key) else {
        return INVALID_STRING;
    };
    let Ok(value) = HeaderValue::try_from(value) else {
        return INVALID_STRING;
    };
    let Some(request) = env
        .data_mut()
        .descriptors
        .get_mut(descriptor)
        .and_then(DescriptorValue::as_request_mut)
    else {
        return INVALID_DESCRIPTOR;
    };
    request.headers.insert(key, value);
    SUCCESS
}

pub(super) fn set_body(
    mut env: FunctionEnvMut<HostState>,
    descriptor: i32,
    pointer: i32,
    length: i32,
) -> i32 {
    if pointer < 0 || length < 0 {
        return INVALID_STRING;
    }
    let Ok(body) = env.data().read_bytes(&env, pointer as u32, length as u32) else {
        return INVALID_STRING;
    };
    let Some(request) = env
        .data_mut()
        .descriptors
        .get_mut(descriptor)
        .and_then(DescriptorValue::as_request_mut)
    else {
        return INVALID_DESCRIPTOR;
    };
    request.body = Some(body);
    SUCCESS
}

pub(super) fn set_timeout(
    mut env: FunctionEnvMut<HostState>,
    descriptor: i32,
    seconds: f64,
) -> i32 {
    if !seconds.is_finite() || seconds <= 0.0 {
        return INVALID_STRING;
    }
    let Some(request) = env
        .data_mut()
        .descriptors
        .get_mut(descriptor)
        .and_then(DescriptorValue::as_request_mut)
    else {
        return INVALID_DESCRIPTOR;
    };
    request.timeout = Some(Duration::from_secs_f64(seconds.min(86_400.0)));
    SUCCESS
}

pub(super) fn send(mut env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    send_one(&mut env, descriptor)
}

pub(super) fn send_all(mut env: FunctionEnvMut<HostState>, pointer: i32, length: i32) -> i32 {
    let Ok(descriptors) = env.data().read_i32s(&env, pointer, length) else {
        return INVALID_DESCRIPTOR;
    };
    let mut has_error = false;
    let mut results = Vec::with_capacity(descriptors.len());
    for descriptor in descriptors {
        let result = send_one(&mut env, descriptor);
        has_error |= result != SUCCESS;
        results.push(result);
    }
    if env.data().write_i32s(&env, pointer, &results).is_err() {
        FAILED_MEMORY_WRITE
    } else if has_error {
        REQUEST_ERROR
    } else {
        SUCCESS
    }
}

pub(super) fn data_len(env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    env.data()
        .descriptors
        .get(descriptor)
        .and_then(DescriptorValue::as_request)
        .and_then(|request| request.response.as_ref())
        .and_then(|response| i32::try_from(response.data.len()).ok())
        .unwrap_or(MISSING_RESPONSE)
}

pub(super) fn read_data(
    env: FunctionEnvMut<HostState>,
    descriptor: i32,
    pointer: i32,
    size: i32,
) -> i32 {
    if pointer < 0 || size < 0 {
        return FAILED_MEMORY_WRITE;
    }
    let Some(bytes) = env
        .data()
        .descriptors
        .get(descriptor)
        .and_then(DescriptorValue::as_request)
        .and_then(|request| request.response.as_ref())
        .map(|response| response.data.as_slice())
    else {
        return MISSING_RESPONSE;
    };
    if size as usize > bytes.len() {
        return FAILED_MEMORY_WRITE;
    }
    env.data()
        .write_bytes(&env, pointer as u32, &bytes[..size as usize])
        .map(|_| SUCCESS)
        .unwrap_or(FAILED_MEMORY_WRITE)
}

pub(super) fn get_image(mut env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    let Some(bytes) = env
        .data()
        .descriptors
        .get(descriptor)
        .and_then(DescriptorValue::as_request)
        .and_then(|request| request.response.as_ref())
        .map(|response| response.data.clone())
    else {
        return MISSING_RESPONSE;
    };
    let Ok(reader) = ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format() else {
        return NOT_AN_IMAGE;
    };
    let Ok(image) = reader.decode() else {
        return NOT_AN_IMAGE;
    };
    env.data_mut()
        .descriptors
        .insert(DescriptorValue::Image(ImageData::from(image.to_rgba8())))
}

pub(super) fn html(mut env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    let Some(response) = env
        .data()
        .descriptors
        .get(descriptor)
        .and_then(DescriptorValue::as_request)
        .and_then(|request| request.response.as_ref())
        .cloned()
    else {
        return MISSING_RESPONSE;
    };
    let Ok(text) = String::from_utf8(response.data) else {
        return INVALID_HTML;
    };
    env.data_mut()
        .descriptors
        .insert(DescriptorValue::HtmlDocument(html::parse(
            &text,
            Some(response.url.as_str()),
        )))
}

pub(super) fn get_status_code(env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    env.data()
        .descriptors
        .get(descriptor)
        .and_then(DescriptorValue::as_request)
        .and_then(|request| request.response.as_ref())
        .map(|response| i32::from(response.status))
        .unwrap_or(MISSING_RESPONSE)
}

pub(super) fn get_url(mut env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    let Some(url) = env
        .data()
        .descriptors
        .get(descriptor)
        .and_then(DescriptorValue::as_request)
        .and_then(|request| request.response.as_ref())
        .map(|response| response.url.to_string())
    else {
        return MISSING_RESPONSE;
    };
    env.data_mut()
        .descriptors
        .insert(DescriptorValue::Encoded(url.into_bytes()))
}

pub(super) fn get_header(
    mut env: FunctionEnvMut<HostState>,
    descriptor: i32,
    pointer: i32,
    length: i32,
) -> i32 {
    let Ok(key) = env.data().read_string(&env, pointer, length) else {
        return INVALID_STRING;
    };
    let Ok(key) = HeaderName::try_from(key) else {
        return INVALID_STRING;
    };
    let Some(value) = env
        .data()
        .descriptors
        .get(descriptor)
        .and_then(DescriptorValue::as_request)
        .and_then(|request| request.response.as_ref())
        .and_then(|response| response.headers.get(&key))
        .map(|value| value.as_bytes().to_vec())
    else {
        return MISSING_DATA;
    };
    env.data_mut()
        .descriptors
        .insert(DescriptorValue::Encoded(value))
}

pub(super) fn set_rate_limit(
    _env: FunctionEnvMut<HostState>,
    _permits: i32,
    _period: i32,
    _unit: i32,
) {
}

fn send_one(env: &mut FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    let Some(request) = env
        .data()
        .descriptors
        .get(descriptor)
        .and_then(DescriptorValue::as_request)
        .cloned()
    else {
        return INVALID_DESCRIPTOR;
    };
    let Some(url) = request.url.clone() else {
        return INVALID_URL;
    };
    let mut builder = env
        .data()
        .http_client
        .request(request.method.as_reqwest(), url.clone())
        .headers(request.headers);
    if let Some(body) = request.body {
        builder = builder.body(body);
    }
    if let Some(timeout) = request.timeout {
        builder = builder.timeout(timeout);
    }
    let Ok(mut response) = builder.send() else {
        return REQUEST_ERROR;
    };
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return REQUEST_ERROR;
    }
    let mut data = Vec::new();
    let mut limited = (&mut response).take((MAX_RESPONSE_BYTES + 1) as u64);
    if limited.read_to_end(&mut data).is_err() || data.len() > MAX_RESPONSE_BYTES {
        return REQUEST_ERROR;
    }
    let result = NetResponse {
        url: response.url().clone(),
        status: response.status().as_u16(),
        headers: response.headers().clone(),
        data,
    };
    let Some(request) = env
        .data_mut()
        .descriptors
        .get_mut(descriptor)
        .and_then(DescriptorValue::as_request_mut)
    else {
        return INVALID_DESCRIPTOR;
    };
    request.response = Some(result);
    SUCCESS
}
