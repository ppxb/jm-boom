mod bridge;
mod host;
mod store;

use super::SourcePackage;
use host::{build_imports, HostState};
use serde::{de::DeserializeOwned, Serialize};
use store::DescriptorValue;
use thiserror::Error;
use wasmer::{Engine, FunctionEnv, Instance, Module, Store, TypedFunction};

#[derive(Clone)]
pub struct CompiledSource {
    source_id: String,
    engine: Engine,
    module: Module,
}

pub struct SourceInstance {
    store: Store,
    instance: Instance,
    environment: FunctionEnv<HostState>,
}

#[derive(Debug, Error)]
pub enum SourceRuntimeError {
    #[error("failed to compile source module: {0}")]
    Compile(String),
    #[error("failed to instantiate source module: {0}")]
    Instantiate(String),
    #[error("missing or invalid source export: {0}")]
    Export(String),
    #[error("source execution failed: {0}")]
    Execution(String),
    #[error("failed to initialize source host: {0}")]
    Host(String),
    #[error("source returned error code {0}")]
    Source(i32),
    #[error("source returned an error: {0}")]
    SourceMessage(String),
    #[error("source result is invalid: {0}")]
    InvalidResult(String),
    #[error("failed to encode source input: {0}")]
    Encode(#[from] postcard::Error),
}

impl CompiledSource {
    pub fn compile(package: &SourcePackage) -> Result<Self, SourceRuntimeError> {
        let engine = Engine::default();
        let module = Module::new(&engine, package.wasm.as_ref())
            .map_err(|error| SourceRuntimeError::Compile(error.to_string()))?;
        Ok(Self {
            source_id: package.manifest.info.id.clone(),
            engine,
            module,
        })
    }

    pub fn instantiate(&self) -> Result<SourceInstance, SourceRuntimeError> {
        SourceInstance::instantiate(self)
    }
}

impl SourceInstance {
    pub fn load(package: &SourcePackage) -> Result<Self, SourceRuntimeError> {
        CompiledSource::compile(package)?.instantiate()
    }

    fn instantiate(compiled: &CompiledSource) -> Result<Self, SourceRuntimeError> {
        let mut store = Store::new(compiled.engine.clone());
        let environment = FunctionEnv::new(&mut store, HostState::new(compiled.source_id.clone())?);
        let imports = build_imports(&mut store, &environment);
        let instance = Instance::new(&mut store, &compiled.module, &imports)
            .map_err(|error| SourceRuntimeError::Instantiate(error.to_string()))?;
        let memory = instance
            .exports
            .get_memory("memory")
            .map_err(|error| SourceRuntimeError::Export(error.to_string()))?
            .clone();
        environment.as_mut(&mut store).memory = Some(memory);

        let start: TypedFunction<(), ()> = instance
            .exports
            .get_typed_function(&store, "start")
            .map_err(|error| SourceRuntimeError::Export(error.to_string()))?;
        start
            .call(&mut store)
            .map_err(|error| SourceRuntimeError::Execution(error.to_string()))?;

        Ok(Self {
            store,
            instance,
            environment,
        })
    }

    pub fn has_export(&self, name: &str) -> bool {
        self.instance.exports.get_extern(name).is_some()
    }

    pub fn store<T: Serialize>(&mut self, value: &T) -> Result<i32, SourceRuntimeError> {
        let encoded = postcard::to_allocvec(value)?;
        Ok(self.store_bytes(encoded))
    }

    pub fn store_bytes(&mut self, bytes: impl Into<Vec<u8>>) -> i32 {
        self.environment
            .as_mut(&mut self.store)
            .descriptors
            .insert(DescriptorValue::Encoded(bytes.into()))
    }

    pub fn read_result<T: DeserializeOwned>(
        &mut self,
        pointer: i32,
    ) -> Result<T, SourceRuntimeError> {
        if pointer < 0 {
            return Err(SourceRuntimeError::Source(pointer));
        }
        if pointer == 0 {
            return Err(SourceRuntimeError::InvalidResult(
                "source returned a null result pointer".into(),
            ));
        }
        let result = self
            .environment
            .as_ref(&self.store)
            .read_result(&self.store, pointer as u32);

        let free: TypedFunction<i32, ()> = self
            .instance
            .exports
            .get_typed_function(&self.store, "free_result")
            .map_err(|error| SourceRuntimeError::Export(error.to_string()))?;
        free.call(&mut self.store, pointer)
            .map_err(|error| SourceRuntimeError::Execution(error.to_string()))?;
        let (is_error, bytes) = result?;
        if is_error {
            return Err(SourceRuntimeError::SourceMessage(
                String::from_utf8(bytes)
                    .map_err(|error| SourceRuntimeError::InvalidResult(error.to_string()))?,
            ));
        }
        postcard::from_bytes(&bytes)
            .map_err(|error| SourceRuntimeError::InvalidResult(error.to_string()))
    }

    pub fn invoke0<T: DeserializeOwned>(&mut self, export: &str) -> Result<T, SourceRuntimeError> {
        let function: TypedFunction<(), i32> = self
            .instance
            .exports
            .get_typed_function(&self.store, export)
            .map_err(|error| SourceRuntimeError::Export(error.to_string()))?;
        let pointer = function
            .call(&mut self.store)
            .map_err(|error| SourceRuntimeError::Execution(error.to_string()))?;
        self.read_result(pointer)
    }

    pub fn invoke1<T: DeserializeOwned>(
        &mut self,
        export: &str,
        argument: i32,
    ) -> Result<T, SourceRuntimeError> {
        let function: TypedFunction<i32, i32> = self
            .instance
            .exports
            .get_typed_function(&self.store, export)
            .map_err(|error| SourceRuntimeError::Export(error.to_string()))?;
        let pointer = function
            .call(&mut self.store, argument)
            .map_err(|error| SourceRuntimeError::Execution(error.to_string()))?;
        self.read_result(pointer)
    }

    pub fn invoke2<T: DeserializeOwned>(
        &mut self,
        export: &str,
        first: i32,
        second: i32,
    ) -> Result<T, SourceRuntimeError> {
        let function: TypedFunction<(i32, i32), i32> = self
            .instance
            .exports
            .get_typed_function(&self.store, export)
            .map_err(|error| SourceRuntimeError::Export(error.to_string()))?;
        let pointer = function
            .call(&mut self.store, first, second)
            .map_err(|error| SourceRuntimeError::Execution(error.to_string()))?;
        self.read_result(pointer)
    }

    pub fn invoke3<T: DeserializeOwned>(
        &mut self,
        export: &str,
        first: i32,
        second: i32,
        third: i32,
    ) -> Result<T, SourceRuntimeError> {
        let function: TypedFunction<(i32, i32, i32), i32> = self
            .instance
            .exports
            .get_typed_function(&self.store, export)
            .map_err(|error| SourceRuntimeError::Export(error.to_string()))?;
        let pointer = function
            .call(&mut self.store, first, second, third)
            .map_err(|error| SourceRuntimeError::Execution(error.to_string()))?;
        self.read_result(pointer)
    }

    pub fn remove_descriptor(&mut self, descriptor: i32) {
        self.environment
            .as_mut(&mut self.store)
            .descriptors
            .remove(descriptor);
    }

    fn create_default_image_request(&mut self, url: &str) -> Result<i32, SourceRuntimeError> {
        host::net::create_get_request(self.environment.as_mut(&mut self.store), url)
            .ok_or_else(|| SourceRuntimeError::Host("invalid image URL".into()))
    }

    fn send_image_request(&mut self, descriptor: i32) -> Result<(), SourceRuntimeError> {
        let result = host::net::send_request(self.environment.as_mut(&mut self.store), descriptor);
        if result == 0 {
            Ok(())
        } else {
            Err(SourceRuntimeError::Execution(format!(
                "image request failed with host code {result}"
            )))
        }
    }

    fn image_response_snapshot(
        &self,
        descriptor: i32,
    ) -> Result<host::net::ResponseSnapshot, SourceRuntimeError> {
        host::net::response_snapshot(self.environment.as_ref(&self.store), descriptor)
            .ok_or_else(|| SourceRuntimeError::Host("missing image response".into()))
    }

    fn image_from_request(&mut self, descriptor: i32) -> Result<i32, SourceRuntimeError> {
        let image =
            host::net::image_from_request(self.environment.as_mut(&mut self.store), descriptor);
        if image >= 0 {
            Ok(image)
        } else {
            Err(SourceRuntimeError::Execution(format!(
                "image decode failed with host code {image}"
            )))
        }
    }

    fn encode_image(&self, descriptor: i32) -> Result<Vec<u8>, SourceRuntimeError> {
        let image = self
            .environment
            .as_ref(&self.store)
            .descriptors
            .get(descriptor)
            .and_then(DescriptorValue::as_image)
            .ok_or_else(|| SourceRuntimeError::Host("processed image is missing".into()))?;
        let encoder = webp::Encoder::from_rgba(image.as_raw(), image.width(), image.height());
        Ok(encoder.encode(80.0).to_vec())
    }
}
