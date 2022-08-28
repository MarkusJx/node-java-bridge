#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]
use crate::jni::ext::library_error::LibraryError;
use crate::sys;

type JniCreateJavaVm = unsafe extern "C" fn(
    pvm: *mut *mut sys::JavaVM,
    penv: *mut *mut std::os::raw::c_void,
    args: *mut std::os::raw::c_void,
) -> sys::jint;

static mut LIBRARY: Option<libloading::Library> = None;
static mut JNI_CREATE_JVM: Option<libloading::Symbol<JniCreateJavaVm>> = None;

pub fn load_library(library_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        if LIBRARY.is_some() {
            return Err(Box::from(LibraryError::new("Library already loaded")));
        }

        LIBRARY = Some(libloading::Library::new(library_path)?);
        JNI_CREATE_JVM = Some(
            LIBRARY
                .as_ref()
                .unwrap()
                .get::<JniCreateJavaVm>("JNI_CreateJavaVM".as_ref())?,
        );
    }

    Ok(())
}

pub fn library_loaded() -> bool {
    unsafe { LIBRARY.is_some() && JNI_CREATE_JVM.is_some() }
}

pub fn get_jni_create_java_vm(
) -> Result<&'static libloading::Symbol<'static, JniCreateJavaVm>, Box<dyn std::error::Error>> {
    unsafe {
        JNI_CREATE_JVM
            .as_ref()
            .ok_or(Box::from(LibraryError::new("The library is not loaded")))
    }
}
