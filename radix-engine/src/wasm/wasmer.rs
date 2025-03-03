use std::sync::{Arc, Mutex};

use crate::model::InvokeError;
use wasmer::{
    imports, Function, HostEnvInitError, Instance, LazyInit, Module, RuntimeError, Store,
    Universal, Val, WasmerEnv,
};
use wasmer_compiler_singlepass::Singlepass;

use crate::types::*;
use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;

pub struct WasmerModule {
    module: Module,
}

pub struct WasmerInstance {
    instance: Instance,
    // Runtime pointer is shared by the instance and every function that requires `env`.
    // It is updated every time the `invoke_export` is called and `Arc` ensures that the
    // update applies to all the owners.
    runtime_ptr: Arc<Mutex<usize>>,
}

#[derive(Clone)]
pub struct WasmerInstanceEnv {
    instance: LazyInit<Instance>,
    runtime_ptr: Arc<Mutex<usize>>,
}

pub struct WasmerEngine {
    store: Store,
    modules: HashMap<Hash, WasmerModule>,
}

pub fn send_value(
    instance: &Instance,
    value: &ScryptoValue,
) -> Result<usize, InvokeError<WasmError>> {
    let slice = &value.raw;
    let n = slice.len();

    let result = instance
        .exports
        .get_function(EXPORT_SCRYPTO_ALLOC)
        .expect("ScryptoAlloc not found")
        .call(&[Val::I32(n as i32)])
        .map_err(|e| {
            let error: InvokeError<WasmError> = e.into();
            error
        })?;

    if let Some(wasmer::Value::I32(ptr)) = result.as_ref().get(0) {
        let ptr = *ptr as usize;
        let memory = instance
            .exports
            .get_memory(EXPORT_MEMORY)
            .map_err(|_| InvokeError::Error(WasmError::MemoryAllocError))?;
        let size = memory.size().bytes().0;
        if size > ptr && size - ptr >= n {
            unsafe {
                let dest = memory.data_ptr().add(ptr + 4);
                ptr::copy(slice.as_ptr(), dest, n);
            }
            return Ok(ptr);
        }
    }

    Err(InvokeError::Error(WasmError::MemoryAllocError))
}

pub fn read_value(instance: &Instance, ptr: usize) -> Result<ScryptoValue, WasmError> {
    let memory = instance
        .exports
        .get_memory(EXPORT_MEMORY)
        .map_err(|_| WasmError::MemoryAccessError)?;
    let size = memory.size().bytes().0;
    if size > ptr && size - ptr >= 4 {
        // read len
        let mut temp = [0u8; 4];
        unsafe {
            let from = memory.data_ptr().add(ptr);
            ptr::copy(from, temp.as_mut_ptr(), 4);
        }
        let n = u32::from_le_bytes(temp) as usize;

        // read value
        if size - ptr - 4 >= (n as usize) {
            // TODO: avoid copying
            let mut temp = Vec::with_capacity(n);
            unsafe {
                let from = memory.data_ptr().add(ptr).add(4);
                ptr::copy(from, temp.as_mut_ptr(), n);
                temp.set_len(n);
            }

            return ScryptoValue::from_slice(&temp).map_err(WasmError::InvalidScryptoValue);
        }
    }

    Err(WasmError::MemoryAccessError)
}

impl WasmerEnv for WasmerInstanceEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.instance.initialize(instance.clone());
        Ok(())
    }
}

impl WasmerModule {
    fn instantiate(&self) -> WasmerInstance {
        // native functions
        fn radix_engine(env: &WasmerInstanceEnv, input_ptr: i32) -> Result<i32, RuntimeError> {
            let instance = unsafe { env.instance.get_unchecked() };
            let input = read_value(&instance, input_ptr as usize)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            let output = {
                let ptr = env
                    .runtime_ptr
                    .lock()
                    .expect("Failed to lock WASM runtime pointer");
                let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };
                runtime
                    .main(input)
                    .map_err(|e| RuntimeError::user(Box::new(e)))?
            };

            send_value(&instance, &output)
                .map(|ptr| ptr as i32)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        fn consume_cost_units(env: &WasmerInstanceEnv, cost_unit: i32) -> Result<(), RuntimeError> {
            let ptr = env
                .runtime_ptr
                .lock()
                .expect("Failed to lock WASM runtime pointer");
            let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };
            runtime
                .consume_cost_units(cost_unit as u32)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        // env
        let env = WasmerInstanceEnv {
            instance: LazyInit::new(),
            runtime_ptr: Arc::new(Mutex::new(0)),
        };

        // imports
        let import_object = imports! {
            MODULE_ENV_NAME => {
                RADIX_ENGINE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), radix_engine),
                CONSUME_COST_UNITS_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), consume_cost_units),
            }
        };

        // instantiate
        let instance =
            Instance::new(&self.module, &import_object).expect("Failed to instantiate WASM module");

        WasmerInstance {
            instance,
            runtime_ptr: env.runtime_ptr,
        }
    }
}

impl From<RuntimeError> for InvokeError<WasmError> {
    fn from(error: RuntimeError) -> Self {
        let e_str = format!("{:?}", error);
        match error.downcast::<InvokeError<WasmError>>() {
            Ok(e) => e,
            _ => InvokeError::Error(WasmError::WasmError(e_str)),
        }
    }
}

impl WasmInstance for WasmerInstance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: &ScryptoValue,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<ScryptoValue, InvokeError<WasmError>> {
        {
            // set up runtime pointer
            let mut guard = self
                .runtime_ptr
                .lock()
                .expect("Failed to lock WASM runtime pointer");
            *guard = runtime as *mut _ as usize;
        }

        let pointer = send_value(&self.instance, args)?;
        let result = self
            .instance
            .exports
            .get_function(func_name)
            .map_err(|_| InvokeError::Error(WasmError::FunctionNotFound))?
            .call(&[Val::I32(pointer as i32)]);

        match result {
            Ok(return_data) => {
                let ptr = return_data
                    .as_ref()
                    .get(0)
                    .ok_or(InvokeError::Error(WasmError::MissingReturnData))?
                    .i32()
                    .ok_or(InvokeError::Error(WasmError::InvalidReturnData))?;
                read_value(&self.instance, ptr as usize).map_err(InvokeError::Error)
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl WasmerEngine {
    pub fn new() -> Self {
        let compiler = Singlepass::new();
        Self {
            store: Store::new(&Universal::new(compiler).engine()),
            modules: HashMap::new(),
        }
    }
}

impl WasmEngine<WasmerInstance> for WasmerEngine {
    fn instantiate(&mut self, code: &[u8]) -> WasmerInstance {
        let code_hash = hash(code);
        self.modules
            .entry(code_hash)
            .or_insert_with(|| WasmerModule {
                module: Module::new(&self.store, code).expect("Failed to parse WASM module"),
            })
            .instantiate()
    }
}
