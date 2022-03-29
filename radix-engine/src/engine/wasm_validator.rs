use wasmi::*;

use crate::engine::*;
use crate::errors::*;

/// Parses a WASM module.
pub fn parse_module(code: &[u8]) -> Result<Module, WasmValidationError> {
    Module::from_buffer(code).map_err(|_| WasmValidationError::InvalidModule)
}

/// Validates a WASM module.
pub fn validate_module(code: &[u8]) -> Result<(), WasmValidationError> {
    // Parse
    let parsed = parse_module(code)?;

    // check floating point
    parsed
        .deny_floating_point()
        .map_err(|_| WasmValidationError::FloatingPointNotAllowed)?;

    // Instantiate
    let instance = ModuleInstance::new(
        &parsed,
        &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
    )
    .map_err(|_| WasmValidationError::InvalidModule)?;

    // Check start function
    if instance.has_start() {
        return Err(WasmValidationError::StartFunctionNotAllowed);
    }
    let module = instance.assert_no_start();

    // Check memory export
    match module.export_by_name("memory") {
        Some(ExternVal::Memory(_)) => {}
        _ => return Err(WasmValidationError::NoValidMemoryExport)
    }

    module.invoke_export("package_init", &[], &mut NopExternals)
        .map_err(|e| WasmValidationError::NoPackageInitExport(e.into()))?;

    Ok(())
}
