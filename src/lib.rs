#![deny(clippy::all)]

mod jni;
mod node;
mod sys;
#[cfg(test)]
mod tests;

#[macro_use]
extern crate napi_derive;
extern crate core;

use crate::node::napi_error::NapiError;
use napi::Status;
use std::error::Error;
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
        .ok_or(Box::<dyn Error>::from("Could not create the library path"))
        .map_err(NapiError::to_napi_error)?
        .to_string())
}
