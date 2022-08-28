use crate::jni::ext;
use crate::jni::ext::library::{library_loaded, load_library};
use crate::jni::java_env::JavaEnv;
use crate::jni::jni_error::JNIError;
use crate::jni::util::util::{jni_error_to_string, parse_jni_version, ResultType};
use crate::jni::vm_ptr::JavaVMPtr;
use crate::sys;
use std::error::Error;
use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// The Java Virtual Machine.
/// This is the main entry point to the JNI interface.
/// It may only be created once. All subsequent calls to
/// [`new`](Self::new) will return an error.
#[derive(Clone)]
pub struct JavaVM {
    ptr: Arc<Mutex<JavaVMPtr>>,
    options: InternalJavaOptions,
}

#[allow(non_snake_case, dead_code)]
struct JavaVMOption {
    optionString: *mut c_char,
    extraInfo: *mut c_void,
}

impl JavaVM {
    /// Create a new Java Virtual Machine.
    pub fn new(
        version: &String,
        library_path: Option<String>,
        args: &Vec<String>,
        options: InternalJavaOptions,
    ) -> ResultType<Self> {
        if !library_loaded() {
            let lib_path = match library_path {
                Some(lib_path) => lib_path,
                None => {
                    let lib = java_locator::locate_jvm_dyn_library()?;
                    let base = Path::new(lib.as_str());
                    let path = base.join(java_locator::get_jvm_dyn_lib_file_name());

                    path.to_str()
                        .ok_or(Box::<dyn Error>::from("Could not create the library path"))?
                        .to_string()
                }
            };

            load_library(lib_path.as_str())?;
        }

        let create_fn = ext::library::get_jni_create_java_vm()?;
        let mut ptr: *mut sys::JavaVM = std::ptr::null_mut();
        let mut env: *mut sys::JNIEnv = std::ptr::null_mut();

        let mut opts = Vec::with_capacity(args.len());
        for opt in args {
            let option_string = CString::new(opt.as_str())?;
            let jvm_opt = JavaVMOption {
                optionString: option_string.into_raw(),
                extraInfo: std::ptr::null_mut(),
            };
            opts.push(jvm_opt);
        }

        let mut vm_args = sys::JavaVMInitArgs {
            version: parse_jni_version(version.as_str())? as i32,
            options: opts.as_mut_ptr() as _,
            nOptions: opts.len() as _,
            ignoreUnrecognized: 0,
        };

        let create_res: i32 = unsafe {
            create_fn(
                &mut ptr,
                &mut env as *mut *mut sys::JNIEnv as *mut *mut std::os::raw::c_void,
                &mut vm_args as *mut sys::JavaVMInitArgs as *mut std::os::raw::c_void,
            )
        };

        if create_res != 0 {
            return Err(JNIError::new(format!(
                "Failed to create JavaVM: {}",
                jni_error_to_string(create_res)
            ))
            .into());
        }

        let ptr = Arc::new(Mutex::new(JavaVMPtr::new(ptr, options)));
        let thread = JavaVM::_attach_thread(&ptr, &options)?;
        ptr.lock()
            .unwrap()
            .set_class_loader(thread.get_system_class_loader()?);

        Ok(Self { ptr, options })
    }

    pub(in crate::jni) fn from_existing(
        ptr: Arc<Mutex<JavaVMPtr>>,
        options: InternalJavaOptions,
    ) -> Self {
        Self { ptr, options }
    }

    pub fn options(&self) -> InternalJavaOptions {
        self.options
    }

    pub fn get_version(&self) -> ResultType<String> {
        let env = self.attach_thread()?;
        Ok(env.get_version()?)
    }

    fn _attach_thread<'a>(
        ptr: &Arc<Mutex<JavaVMPtr>>,
        options: &InternalJavaOptions,
    ) -> ResultType<JavaEnv<'a>> {
        let mut env: *mut sys::JNIEnv = std::ptr::null_mut();
        let jvm_ptr = ptr.lock().map_err(|e| JNIError::new(e.to_string()))?;

        let mut create_result = unsafe {
            jvm_ptr.methods().GetEnv.unwrap()(
                jvm_ptr.vm(),
                &mut env as *mut *mut sys::JNIEnv as *mut *mut std::os::raw::c_void,
                sys::JNI_VERSION_1_8 as i32,
            )
        };

        if create_result == sys::JNI_EDETACHED {
            let method = unsafe {
                if options.use_daemon_threads {
                    jvm_ptr.methods().AttachCurrentThreadAsDaemon.unwrap()
                } else {
                    jvm_ptr.methods().AttachCurrentThread.unwrap()
                }
            };

            create_result = unsafe {
                method(
                    jvm_ptr.vm(),
                    &mut env as *mut *mut sys::JNIEnv as *mut *mut std::os::raw::c_void,
                    std::ptr::null_mut(),
                )
            };

            drop(jvm_ptr);
            if create_result != sys::JNI_OK as i32 {
                return Err(JNIError::new(format!(
                    "Failed to attach thread: {}",
                    jni_error_to_string(create_result)
                ))
                .into());
            }

            Ok(JavaEnv::new(ptr.clone(), options.clone(), env))
        } else if create_result == sys::JNI_OK as i32 {
            drop(jvm_ptr);
            Ok(JavaEnv::new(ptr.clone(), options.clone(), env))
        } else {
            Err(JNIError::new(format!(
                "Failed to attach thread: {}",
                jni_error_to_string(create_result)
            ))
            .into())
        }
    }

    pub fn attach_thread<'a>(&self) -> ResultType<JavaEnv<'a>> {
        JavaVM::_attach_thread(&self.ptr, &self.options)
    }
}

unsafe impl Send for JavaVM {}

#[derive(Copy, Clone)]
pub struct InternalJavaOptions {
    pub use_daemon_threads: bool,
}
