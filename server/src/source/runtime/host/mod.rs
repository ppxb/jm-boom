use super::store::DescriptorStore;
use crate::source::runtime::SourceRuntimeError;
use wasmer::{imports, Function, FunctionEnv, FunctionEnvMut, Imports, Memory, Store, WasmPtr};

pub struct HostState {
    source_id: String,
    pub memory: Option<Memory>,
    pub descriptors: DescriptorStore,
}

impl HostState {
    pub fn new(source_id: String) -> Self {
        Self {
            source_id,
            memory: None,
            descriptors: DescriptorStore::new(),
        }
    }

    fn read_bytes(
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
        let bytes = WasmPtr::<u8>::new(pointer)
            .slice(&view, length)
            .map_err(|error| SourceRuntimeError::InvalidResult(error.to_string()))?;
        bytes
            .iter()
            .map(|byte| {
                byte.read()
                    .map_err(|error| SourceRuntimeError::InvalidResult(error.to_string()))
            })
            .collect()
    }

    fn write_bytes(
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

    fn read_string(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: u32,
        length: u32,
    ) -> Result<String, SourceRuntimeError> {
        String::from_utf8(self.read_bytes(store, pointer, length)?)
            .map_err(|error| SourceRuntimeError::InvalidResult(error.to_string()))
    }

    fn read_u32(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: u32,
    ) -> Result<u32, SourceRuntimeError> {
        let bytes = self.read_bytes(store, pointer, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
    }

    pub fn read_result_bytes(
        &self,
        store: &impl wasmer::AsStoreRef,
        pointer: u32,
    ) -> Result<Vec<u8>, SourceRuntimeError> {
        let length = self.read_u32(store, pointer)?;
        if length < 8 {
            return Err(SourceRuntimeError::InvalidResult(
                "result length is smaller than its header".into(),
            ));
        }
        self.read_bytes(store, pointer + 8, length - 8)
    }
}

pub fn build_imports(store: &mut Store, env: &FunctionEnv<HostState>) -> Imports {
    imports! {
        "env" => {
            "print" => Function::new_typed_with_env(store, env, print),
            "send_partial_result" => Function::new_typed_with_env(store, env, send_partial_result),
        },
        "std" => {
            "print" => Function::new_typed_with_env(store, env, print),
            "abort" => Function::new_typed_with_env(store, env, abort),
            "destroy" => Function::new_typed_with_env(store, env, destroy),
            "buffer_len" => Function::new_typed_with_env(store, env, buffer_len),
            "read_buffer" => Function::new_typed_with_env(store, env, read_buffer),
            "current_date" => Function::new_typed_with_env(store, env, current_date),
            "parse_date" => Function::new_typed_with_env(store, env, parse_date),
        },
        "net" => {
            "init" => Function::new_typed_with_env(store, env, stub_i32_i32),
            "send" => Function::new_typed_with_env(store, env, stub_i32_i32),
            "send_all" => Function::new_typed_with_env(store, env, stub_i32_i32_i32),
            "set_url" => Function::new_typed_with_env(store, env, stub_i32_i32_i32_i32),
            "set_header" => Function::new_typed_with_env(store, env, stub_five_i32),
            "set_body" => Function::new_typed_with_env(store, env, stub_i32_i32_i32_i32),
            "data_len" => Function::new_typed_with_env(store, env, stub_i32_i32),
            "read_data" => Function::new_typed_with_env(store, env, stub_i32_i32_i32_i32),
            "html" => Function::new_typed_with_env(store, env, stub_i32_i32),
        },
        "html" => {
            "attr" => Function::new_typed_with_env(store, env, stub_i32_i32_i32_i32),
            "html" => Function::new_typed_with_env(store, env, stub_i32_i32),
            "text" => Function::new_typed_with_env(store, env, stub_i32_i32),
            "get" => Function::new_typed_with_env(store, env, stub_i32_i32_i32),
            "select" => Function::new_typed_with_env(store, env, stub_i32_i32_i32_i32),
            "select_first" => Function::new_typed_with_env(store, env, stub_i32_i32_i32_i32),
            "size" => Function::new_typed_with_env(store, env, stub_i32_i32),
        },
        "defaults" => {
            "get" => Function::new_typed_with_env(store, env, stub_i32_i32_i32),
            "set" => Function::new_typed_with_env(store, env, stub_i32_i32_i32_i32_i32),
        },
        "canvas" => {
            "get_image_width" => Function::new_typed_with_env(store, env, stub_i32_f32),
            "get_image_height" => Function::new_typed_with_env(store, env, stub_i32_f32),
            "new_context" => Function::new_typed_with_env(store, env, stub_f32_f32_i32),
            "copy_image" => Function::new_typed_with_env(store, env, stub_copy_image),
            "get_image" => Function::new_typed_with_env(store, env, stub_i32_i32),
        },
    }
}

fn print(env: FunctionEnvMut<HostState>, pointer: i32, length: i32) {
    if pointer < 0 || length <= 0 {
        return;
    }
    match env
        .data()
        .read_string(&env, pointer as u32, length as u32)
    {
        Ok(message) => tracing::debug!(source = %env.data().source_id, %message, "漫画源日志"),
        Err(error) => tracing::warn!(source = %env.data().source_id, %error, "无法读取漫画源日志"),
    }
}

fn send_partial_result(_env: FunctionEnvMut<HostState>, _pointer: i32) {}

fn abort(env: FunctionEnvMut<HostState>) {
    tracing::warn!(source = %env.data().source_id, "漫画源主动终止执行");
}

fn destroy(mut env: FunctionEnvMut<HostState>, descriptor: i32) {
    env.data_mut().descriptors.remove(descriptor);
}

fn buffer_len(env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    env.data()
        .descriptors
        .get(descriptor)
        .and_then(|value| i32::try_from(value.as_bytes().len()).ok())
        .unwrap_or(-1)
}

fn read_buffer(mut env: FunctionEnvMut<HostState>, descriptor: i32, pointer: i32, size: i32) -> i32 {
    if pointer < 0 || size < 0 {
        return -3;
    }
    let Some(bytes) = env
        .data()
        .descriptors
        .get(descriptor)
        .map(|value| value.as_bytes().to_vec())
    else {
        return -1;
    };
    if size as usize > bytes.len() {
        return -3;
    }
    env.data_mut()
        .write_bytes(&env, pointer as u32, &bytes[..size as usize])
        .map(|_| 0)
        .unwrap_or(-3)
}

fn current_date(_env: FunctionEnvMut<HostState>) -> f64 {
    chrono::Utc::now().timestamp() as f64
}

#[allow(clippy::too_many_arguments)]
fn parse_date(
    _env: FunctionEnvMut<HostState>,
    _date_ptr: i32,
    _date_len: i32,
    _format_ptr: i32,
    _format_len: i32,
    _locale_ptr: i32,
    _locale_len: i32,
    _timezone_ptr: i32,
    _timezone_len: i32,
) -> f64 {
    -5.0
}

fn stub_i32_i32(_env: FunctionEnvMut<HostState>, _a: i32) -> i32 { -1 }
fn stub_i32_i32_i32(_env: FunctionEnvMut<HostState>, _a: i32, _b: i32) -> i32 { -1 }
fn stub_i32_i32_i32_i32(_env: FunctionEnvMut<HostState>, _a: i32, _b: i32, _c: i32) -> i32 { -1 }
fn stub_i32_i32_i32_i32_i32(_env: FunctionEnvMut<HostState>, _a: i32, _b: i32, _c: i32, _d: i32) -> i32 { -1 }
fn stub_five_i32(_env: FunctionEnvMut<HostState>, _a: i32, _b: i32, _c: i32, _d: i32, _e: i32) -> i32 { -1 }
fn stub_i32_f32(_env: FunctionEnvMut<HostState>, _a: i32) -> f32 { -1.0 }
fn stub_f32_f32_i32(_env: FunctionEnvMut<HostState>, _a: f32, _b: f32) -> i32 { -1 }

#[allow(clippy::too_many_arguments)]
fn stub_copy_image(
    _env: FunctionEnvMut<HostState>,
    _context: i32,
    _image: i32,
    _src_x: f32,
    _src_y: f32,
    _src_width: f32,
    _src_height: f32,
    _dst_x: f32,
    _dst_y: f32,
    _dst_width: f32,
    _dst_height: f32,
) -> i32 {
    -1
}
