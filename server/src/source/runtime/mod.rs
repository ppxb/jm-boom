mod host;
mod store;

use super::SourcePackage;
use host::{build_imports, HostState};
use serde::{de::DeserializeOwned, Serialize};
use store::DescriptorValue;
use thiserror::Error;
use wasmer::{FunctionEnv, Instance, Module, Store, TypedFunction};

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
    #[error("source returned error code {0}")]
    Source(i32),
    #[error("source result is invalid: {0}")]
    InvalidResult(String),
    #[error("failed to encode source input: {0}")]
    Encode(#[from] postcard::Error),
}

impl SourceInstance {
    pub fn load(package: &SourcePackage) -> Result<Self, SourceRuntimeError> {
        let mut store = Store::default();
        let module = Module::new(&store, package.wasm.as_ref())
            .map_err(|error| SourceRuntimeError::Compile(error.to_string()))?;
        let environment = FunctionEnv::new(
            &mut store,
            HostState::new(package.manifest.info.id.clone()),
        );
        let imports = build_imports(&mut store, &environment);
        let instance = Instance::new(&mut store, &module, &imports)
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
        self.instance.exports.get_extern(name).is_ok()
    }

    pub fn store<T: Serialize>(&mut self, value: &T) -> Result<i32, SourceRuntimeError> {
        let encoded = postcard::to_allocvec(value)?;
        Ok(self
            .environment
            .as_mut(&mut self.store)
            .descriptors
            .insert(DescriptorValue::Encoded(encoded)))
    }

    pub fn read_result<T: DeserializeOwned>(
        &mut self,
        pointer: i32,
    ) -> Result<T, SourceRuntimeError> {
        if pointer < 0 {
            return Err(SourceRuntimeError::Source(pointer));
        }
        let bytes = self
            .environment
            .as_ref(&self.store)
            .read_result_bytes(&self.store, pointer as u32)?;
        let result = postcard::from_bytes(&bytes)
            .map_err(|error| SourceRuntimeError::InvalidResult(error.to_string()))?;

        let free: TypedFunction<i32, ()> = self
            .instance
            .exports
            .get_typed_function(&self.store, "free_result")
            .map_err(|error| SourceRuntimeError::Export(error.to_string()))?;
        free.call(&mut self.store, pointer)
            .map_err(|error| SourceRuntimeError::Execution(error.to_string()))?;
        Ok(result)
    }
}
