pub(super) mod canvas;
pub(super) mod defaults;
pub(super) mod env;
pub(super) mod html;
pub(super) mod net;
pub(super) mod std;

use reqwest::blocking::Client;
use wasmer::{imports, Function, FunctionEnv, Imports, Memory, Store, WasmPtr};

use super::store::DescriptorStore;
use crate::source::runtime::SourceRuntimeError;

const MAX_RESULT_BYTES: u32 = 64 * 1024 * 1024;
const MAX_DESCRIPTOR_ARRAY_LENGTH: i32 = 16_384;

pub(super) struct HostState {
    source_id: String,
    pub(super) memory: Option<Memory>,
    pub(super) descriptors: DescriptorStore,
    defaults: defaults::UserDefaults,
    partial_results: Vec<Vec<u8>>,
    http_client: Client,
}

// Wasmer requires host environments to satisfy Send + Sync even though a source
// instance is executed on one thread and never shared across stores. The HTML
// tree is therefore guarded by the source instance's single-threaded ownership.
unsafe impl Send for HostState {}
unsafe impl Sync for HostState {}

impl HostState {
    pub(super) fn new(source_id: String) -> Result<Self, SourceRuntimeError> {
        let http_client = Client::builder()
            .cookie_store(true)
            .user_agent(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) \
                 AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1",
            )
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|error| SourceRuntimeError::Host(error.to_string()))?;
        Ok(Self {
            source_id,
            memory: None,
            descriptors: DescriptorStore::new(),
            defaults: defaults::UserDefaults::new(),
            partial_results: Vec::new(),
            http_client,
        })
    }

    pub(super) fn read_bytes(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: u32,
        length: u32,
    ) -> Result<Vec<u8>, SourceRuntimeError> {
        let memory = self
            .memory
            .as_ref()
            .ok_or_else(|| SourceRuntimeError::InvalidResult("memory is not initialized".into()))?;
        let view = memory.view(store);
        WasmPtr::<u8>::new(pointer)
            .slice(&view, length)
            .map_err(|error| SourceRuntimeError::InvalidResult(error.to_string()))?
            .iter()
            .map(|byte| {
                byte.read()
                    .map_err(|error| SourceRuntimeError::InvalidResult(error.to_string()))
            })
            .collect()
    }

    pub(super) fn write_bytes(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: u32,
        bytes: &[u8],
    ) -> Result<(), SourceRuntimeError> {
        let memory = self
            .memory
            .as_ref()
            .ok_or_else(|| SourceRuntimeError::InvalidResult("memory is not initialized".into()))?;
        memory
            .view(store)
            .write(u64::from(pointer), bytes)
            .map_err(|error| SourceRuntimeError::InvalidResult(error.to_string()))
    }

    pub(super) fn read_string(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: i32,
        length: i32,
    ) -> Result<String, SourceRuntimeError> {
        if pointer < 0 || length < 0 {
            return Err(SourceRuntimeError::InvalidResult(
                "invalid string range".into(),
            ));
        }
        String::from_utf8(self.read_bytes(store, pointer as u32, length as u32)?)
            .map_err(|error| SourceRuntimeError::InvalidResult(error.to_string()))
    }

    pub(super) fn read_item_bytes(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: i32,
    ) -> Result<Vec<u8>, SourceRuntimeError> {
        if pointer < 0 {
            return Err(SourceRuntimeError::InvalidResult(
                "invalid item pointer".into(),
            ));
        }
        let length = self.read_u32(store, pointer as u32)?;
        if !(8..=MAX_RESULT_BYTES).contains(&length) {
            return Err(SourceRuntimeError::InvalidResult(
                "invalid item length".into(),
            ));
        }
        self.read_bytes(store, pointer as u32 + 8, length - 8)
    }

    pub(super) fn read_i32s(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: i32,
        length: i32,
    ) -> Result<Vec<i32>, SourceRuntimeError> {
        if pointer < 0 || !(0..=MAX_DESCRIPTOR_ARRAY_LENGTH).contains(&length) {
            return Err(SourceRuntimeError::InvalidResult(
                "invalid descriptor range".into(),
            ));
        }
        let byte_length = (length as u32).checked_mul(4).ok_or_else(|| {
            SourceRuntimeError::InvalidResult("descriptor array is too large".into())
        })?;
        let bytes = self.read_bytes(store, pointer as u32, byte_length)?;
        Ok(bytes
            .chunks_exact(4)
            .map(|chunk| i32::from_le_bytes(chunk.try_into().expect("four bytes")))
            .collect())
    }

    pub(super) fn write_i32s(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: i32,
        values: &[i32],
    ) -> Result<(), SourceRuntimeError> {
        if pointer < 0 {
            return Err(SourceRuntimeError::InvalidResult(
                "invalid descriptor pointer".into(),
            ));
        }
        let bytes = values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>();
        self.write_bytes(store, pointer as u32, &bytes)
    }

    pub(super) fn read_result(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: u32,
    ) -> Result<(bool, Vec<u8>), SourceRuntimeError> {
        let header = self.read_bytes(store, pointer, 4)?;
        let marker = i32::from_le_bytes(header.try_into().expect("four bytes"));
        if marker == -1 {
            let message_length = self.read_u32(store, pointer + 8)?;
            if !(12..=MAX_RESULT_BYTES).contains(&message_length) {
                return Err(SourceRuntimeError::InvalidResult(
                    "invalid source error result".into(),
                ));
            }
            return Ok((
                true,
                self.read_bytes(store, pointer + 12, message_length - 12)?,
            ));
        }
        let length = u32::try_from(marker)
            .ok()
            .filter(|length| (8..=MAX_RESULT_BYTES).contains(length))
            .ok_or_else(|| SourceRuntimeError::InvalidResult("invalid result length".into()))?;
        Ok((false, self.read_bytes(store, pointer + 8, length - 8)?))
    }

    fn read_u32(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: u32,
    ) -> Result<u32, SourceRuntimeError> {
        let bytes = self.read_bytes(store, pointer, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
    }
}

pub(super) fn build_imports(store: &mut Store, env: &FunctionEnv<HostState>) -> Imports {
    imports! {
        "env" => {
            "print" => Function::new_typed_with_env(store, env, env::print),
            "abort" => Function::new_typed_with_env(store, env, env::abort),
            "send_partial_result" => Function::new_typed_with_env(store, env, env::send_partial_result),
        },
        "std" => {
            "print" => Function::new_typed_with_env(store, env, env::print),
            "abort" => Function::new_typed_with_env(store, env, env::abort),
            "destroy" => Function::new_typed_with_env(store, env, std::destroy),
            "buffer_len" => Function::new_typed_with_env(store, env, std::buffer_len),
            "read_buffer" => Function::new_typed_with_env(store, env, std::read_buffer),
            "current_date" => Function::new_typed_with_env(store, env, std::current_date),
            "utc_offset" => Function::new_typed_with_env(store, env, std::utc_offset),
            "parse_date" => Function::new_typed_with_env(store, env, std::parse_date),
        },
        "net" => {
            "init" => Function::new_typed_with_env(store, env, net::init),
            "send" => Function::new_typed_with_env(store, env, net::send),
            "send_all" => Function::new_typed_with_env(store, env, net::send_all),
            "set_url" => Function::new_typed_with_env(store, env, net::set_url),
            "set_header" => Function::new_typed_with_env(store, env, net::set_header),
            "set_body" => Function::new_typed_with_env(store, env, net::set_body),
            "set_timeout" => Function::new_typed_with_env(store, env, net::set_timeout),
            "data_len" => Function::new_typed_with_env(store, env, net::data_len),
            "read_data" => Function::new_typed_with_env(store, env, net::read_data),
            "get_image" => Function::new_typed_with_env(store, env, net::get_image),
            "get_status_code" => Function::new_typed_with_env(store, env, net::get_status_code),
            "get_url" => Function::new_typed_with_env(store, env, net::get_url),
            "get_header" => Function::new_typed_with_env(store, env, net::get_header),
            "html" => Function::new_typed_with_env(store, env, net::html),
            "set_rate_limit" => Function::new_typed_with_env(store, env, net::set_rate_limit),
        },
        "html" => {
            "attr" => Function::new_typed_with_env(store, env, html::attr),
            "get" => Function::new_typed_with_env(store, env, html::get),
            "select" => Function::new_typed_with_env(store, env, html::select),
            "select_first" => Function::new_typed_with_env(store, env, html::select_first),
            "size" => Function::new_typed_with_env(store, env, html::size),
            "text" => Function::new_typed_with_env(store, env, html::text),
            "html" => Function::new_typed_with_env(store, env, html::html),
        },
        "defaults" => {
            "get" => Function::new_typed_with_env(store, env, defaults::get),
            "set" => Function::new_typed_with_env(store, env, defaults::set),
        },
        "canvas" => {
            "new_context" => Function::new_typed_with_env(store, env, canvas::new_context),
            "copy_image" => Function::new_typed_with_env(store, env, canvas::copy_image),
            "get_image" => Function::new_typed_with_env(store, env, canvas::get_image),
            "get_image_width" => Function::new_typed_with_env(store, env, canvas::get_image_width),
            "get_image_height" => Function::new_typed_with_env(store, env, canvas::get_image_height),
        },
    }
}
