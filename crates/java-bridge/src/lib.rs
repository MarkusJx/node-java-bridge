#![deny(clippy::all)]

mod java;
mod logging;
mod node;
#[cfg(test)]
mod tests;

#[macro_use]
extern crate napi_derive;
extern crate core;

use napi::Status;
use node::helpers::napi_error::StrIntoNapiError;
use std::path::Path;

/// Get the path to the jvm.(dll|so|dylib) file.
/// Throws an error if the library could not be found.
#[napi]
pub fn get_java_lib_path() -> napi::Result<String> {
    let lib = java_locator::locate_jvm_dyn_library()
        .map_err(|e| napi::Error::new(Status::Unknown, e.to_string()))?;
    let base = Path::new(lib.as_str());
    let path = base.join(java_locator::get_jvm_dyn_lib_file_name());

    Ok(path
        .to_str()
        .ok_or("Could not create the library path".into_napi_err())?
        .to_string())
}
