use crate::jni::java_env::JavaEnv;
use crate::jni::java_vm::{InternalJavaOptions, JavaVM};
use crate::jni::objects::class::JavaClass;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::object::GlobalJavaObject;
use crate::jni::objects::string::JavaString;
use crate::jni::objects::value::JavaBoolean;
use crate::jni::util::util::ResultType;
use crate::node::napi_error::MapToNapiError;
use crate::sys;
use lazy_static::lazy_static;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::{Env, JsFunction, Status};
use std::sync::{Mutex, MutexGuard};

static mut STDOUT_CALLBACK: Mutex<Option<ThreadsafeFunction<String>>> = Mutex::new(None);
static mut STDERR_CALLBACK: Mutex<Option<ThreadsafeFunction<String>>> = Mutex::new(None);

lazy_static! {
    static ref STDOUT_OWNER: Mutex<StdoutOwner> = Mutex::new(StdoutOwner::new());
}

static OPTIONS: Mutex<Option<InternalJavaOptions>> = Mutex::new(None);

struct StdoutOwner {
    current_owner: Option<u32>,
    last_owner: u32,
}

impl StdoutOwner {
    fn new() -> Self {
        StdoutOwner {
            current_owner: None,
            last_owner: 0,
        }
    }

    fn own(&mut self) -> u32 {
        self.last_owner += 1;
        self.current_owner = Some(self.last_owner);

        self.last_owner
    }

    fn owns(&self, id: u32) -> bool {
        self.current_owner == Some(id)
    }

    fn release(&mut self) {
        self.current_owner = None;
    }
}

#[no_mangle]
#[allow(non_snake_case, dead_code)]
pub extern "system" fn Java_io_github_markusjx_bridge_StdoutRedirect_00024CallbackOutputStream_writeLine(
    env: *mut sys::JNIEnv,
    _class: sys::jobject,
    line: sys::jstring,
    is_stdout: sys::jboolean,
) {
    let is_stdout = is_stdout != 0;

    let env = unsafe { JavaEnv::from_raw(env, OPTIONS.lock().unwrap().unwrap()) };
    let string = unsafe { JavaString::from_raw(&env, line) };

    if is_stdout {
        let callback = unsafe { STDOUT_CALLBACK.lock().unwrap() };
        if let Some(callback) = callback.as_ref() {
            callback.call(
                string.to_string().map_napi_err(),
                ThreadsafeFunctionCallMode::NonBlocking,
            );
        }
    } else {
        let callback = unsafe { STDERR_CALLBACK.lock().unwrap() };
        if let Some(callback) = callback.as_ref() {
            callback.call(
                string.to_string().map_napi_err(),
                ThreadsafeFunctionCallMode::NonBlocking,
            );
        }
    }
}

fn map_callback(
    env: &Env,
    callback: &Option<JsFunction>,
    func: &mut MutexGuard<Option<ThreadsafeFunction<String>>>,
) -> napi::Result<()> {
    if let Some(callback) = callback {
        func.replace(env.create_threadsafe_function(&callback, 0, |ctx| {
            Ok(vec![ctx.env.create_string_from_std(ctx.value)?])
        })?);
    } else {
        func.take();
    }

    Ok(())
}

#[napi]
pub struct StdoutRedirect {
    class_instance: GlobalJavaObject,
    vm: JavaVM,
    id: u32,
}

#[napi]
impl StdoutRedirect {
    pub fn new(
        env: Env,
        j_env: &JavaEnv,
        vm: JavaVM,
        stdout_callback: Option<JsFunction>,
        stderr_callback: Option<JsFunction>,
    ) -> ResultType<Self> {
        let mut owner = STDOUT_OWNER.lock().unwrap();
        let mut options = OPTIONS.lock().unwrap();
        if options.is_none() {
            options.replace(vm.options());
        }
        drop(options);

        let class_instance = set_stdout_callbacks(
            env,
            j_env,
            &stdout_callback,
            &stderr_callback,
            stdout_callback.is_some(),
            stderr_callback.is_some(),
            None,
        )?;

        Ok(Self {
            class_instance,
            vm,
            id: owner.own(),
        })
    }

    #[napi]
    pub fn on(
        &mut self,
        env: Env,
        event: String,
        #[napi(ts_arg_type = "((...args: any[]) => any) | null")] callback: Option<JsFunction>,
    ) -> napi::Result<()> {
        let owner = STDOUT_OWNER.lock().unwrap();
        if !owner.owns(self.id) {
            return Err(napi::Error::new(
                Status::Unknown,
                "StdoutRedirect is not owned by current class".to_string(),
            ));
        }

        self.class_instance = match event.as_str() {
            "stdout" => {
                let other_set = unsafe { STDERR_CALLBACK.lock().unwrap().is_some() };
                let j_env = self.vm.attach_thread().map_napi_err()?;
                set_stdout_callbacks(
                    env,
                    &j_env,
                    &callback,
                    &None,
                    callback.is_some(),
                    other_set,
                    Some(&self.class_instance),
                )
                .map_napi_err()?
            }
            "stderr" => {
                let other_set = unsafe { STDOUT_CALLBACK.lock().unwrap().is_some() };
                let j_env = self.vm.attach_thread().map_napi_err()?;
                set_stdout_callbacks(
                    env,
                    &j_env,
                    &None,
                    &callback,
                    other_set,
                    callback.is_some(),
                    Some(&self.class_instance),
                )
                .map_napi_err()?
            }
            _ => {
                return Err(napi::Error::new(
                    Status::InvalidArg,
                    "Invalid event name".to_string(),
                ));
            }
        };

        Ok(())
    }

    #[napi]
    pub fn reset(&self) -> napi::Result<()> {
        let mut owner = STDOUT_OWNER.lock().unwrap();
        if !owner.owns(self.id) {
            return Err(napi::Error::new(
                Status::Unknown,
                "StdoutRedirect is not owned by current class".to_string(),
            ));
        }

        let j_env = self.vm.attach_thread().map_napi_err()?;
        reset_stdout_callbacks(&j_env, Some(&self.class_instance)).map_napi_err()?;

        owner.release();
        Ok(())
    }
}

impl Drop for StdoutRedirect {
    fn drop(&mut self) {
        self.reset().ok();
    }
}

fn set_stdout_callbacks(
    env: Env,
    j_env: &JavaEnv,
    stdout_callback: &Option<JsFunction>,
    stderr_callback: &Option<JsFunction>,
    stdout_set: bool,
    stderr_set: bool,
    java_class: Option<&GlobalJavaObject>,
) -> ResultType<GlobalJavaObject> {
    reset_stdout_callbacks(j_env, java_class)?;

    let mut stdout = unsafe { STDOUT_CALLBACK.lock().unwrap() };
    let mut stderr = unsafe { STDERR_CALLBACK.lock().unwrap() };

    map_callback(&env, stdout_callback, &mut stdout)?;
    map_callback(&env, stderr_callback, &mut stderr)?;

    let class = JavaClass::by_java_name(
        "io.github.markusjx.bridge.StdoutRedirect".to_string(),
        &j_env,
    )?;
    let constructor = class.get_constructor("(ZZ)V")?;

    let instance = constructor.new_instance(
        &j_env,
        vec![
            Box::new(&JavaBoolean::new(stdout_set)),
            Box::new(&JavaBoolean::new(stderr_set)),
        ],
    )?;

    GlobalJavaObject::try_from(instance)
}

fn reset_stdout_callbacks(env: &JavaEnv, java_class: Option<&GlobalJavaObject>) -> ResultType<()> {
    unsafe {
        STDOUT_CALLBACK.lock().unwrap().take();
        STDERR_CALLBACK.lock().unwrap().take();
    }

    if let Some(java_class) = java_class {
        let class =
            JavaClass::by_java_name("io.github.markusjx.bridge.StdoutRedirect".to_string(), &env)?;
        let reset = class.get_void_method("reset", "()V")?;

        reset.call(JavaObject::from(java_class), vec![])?;
    }

    Ok(())
}
