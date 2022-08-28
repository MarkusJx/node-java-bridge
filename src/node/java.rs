use crate::jni::java_vm::{InternalJavaOptions, JavaVM};
use crate::jni::objects::class::JavaClass;
use crate::jni::objects::string::JavaString;
use crate::jni::util::util::ResultType;
use crate::node::java_class_instance::JavaClassInstance;
use crate::node::java_class_proxy::JavaClassProxy;
use crate::node::java_interface_proxy::JavaInterfaceProxy;
use crate::node::java_options::JavaOptions;
use crate::node::napi_error::{MapToNapiError, NapiError};
use crate::node::stdout_redirect::StdoutRedirect;
use futures::future;
use lazy_static::lazy_static;
use napi::{Env, JsFunction, JsObject, JsString, JsUnknown};
use process_path::get_executable_path;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref CACHED_CLASSES: Mutex<HashMap<String, Arc<JavaClassProxy>>> =
        Mutex::new(HashMap::new());
}

pub fn get_class_proxy(vm: &JavaVM, class_name: String) -> ResultType<Arc<JavaClassProxy>> {
    let mut cached_classes = CACHED_CLASSES.lock().unwrap();
    if cached_classes.contains_key(class_name.as_str()) {
        Ok(cached_classes.get(class_name.as_str()).unwrap().clone())
    } else {
        let proxy = Arc::new(JavaClassProxy::new(vm.clone(), class_name.clone())?);
        cached_classes.insert(class_name, proxy.clone());

        Ok(proxy)
    }
}

/// The main java class.
/// This should only be created once per process.
/// Any other attempts to create a new jvm instance will fail.
#[napi]
pub struct Java {
    root_vm: JavaVM,
    wanted_version: String,
    loaded_jars: Vec<String>,
}

#[napi]
impl Java {
    /// Create a new JVM instance.
    /// @param libPath The path to jvm.(dll|so|dylib)
    /// @param version The JVM version to use.
    /// @param opts The JVM options to use.
    #[napi(constructor)]
    pub fn new(
        lib_path: Option<String>,
        version: Option<String>,
        opts: Option<Vec<String>>,
        java_options: Option<JavaOptions>,
        java_lib_path: String,
        native_lib_path: String,
    ) -> napi::Result<Self> {
        let ver = version.unwrap_or("1.8".to_string());

        let path = get_executable_path();
        match path {
            None => println!("The process path could not be determined"),
            Some(path) => println!("{:?}", path),
        }

        let root_vm = JavaVM::new(
            &ver,
            lib_path,
            &opts.unwrap_or(vec![]),
            InternalJavaOptions::from(java_options),
        )
        .map_err(NapiError::to_napi_error)?;

        let env = root_vm.attach_thread().map_napi_err()?;
        env.append_class_path(&vec![java_lib_path]).map_napi_err()?;
        let native_library_class =
            JavaClass::by_java_name("io.github.markusjx.bridge.NativeLibrary".to_string(), &env)
                .map_napi_err()?;

        let load_library = native_library_class
            .get_static_void_method("loadLibrary", "(Ljava/lang/String;)V")
            .map_napi_err()?;
        load_library
            .call(vec![Box::new(
                &JavaString::try_from(native_lib_path, &env).map_napi_err()?,
            )])
            .map_napi_err()?;

        Ok(Self {
            root_vm,
            wanted_version: ver,
            loaded_jars: vec![],
        })
    }

    /// Import a java class
    /// Will import the class and parse all of its methods and fields.
    /// The imported class will be cached for future use.
    #[napi(ts_return_type = "object")]
    pub fn import_class(&mut self, env: Env, class_name: String) -> napi::Result<JsFunction> {
        let proxy = get_class_proxy(&self.root_vm, class_name).map_err(NapiError::to_napi_error)?;
        JavaClassInstance::create_class_instance(&env, proxy)
    }

    /// Import a java class (async)
    /// Will return a promise that resolves to the class instance.
    /// @see importClass
    #[napi(ts_return_type = "Promise<object>")]
    pub fn import_class_async(
        &'static mut self,
        env: Env,
        class_name: String,
    ) -> napi::Result<JsObject> {
        env.execute_tokio_future(
            future::lazy(|_| {
                get_class_proxy(&self.root_vm, class_name).map_err(NapiError::to_napi_error)
            }),
            |&mut env, proxy| JavaClassInstance::create_class_instance(&env, proxy),
        )
    }

    /// Get the wanted JVM version.
    /// This may not match the actual JVM version.
    #[napi(getter)]
    pub fn wanted_version(&self) -> String {
        self.wanted_version.to_string()
    }

    /// Get the actual JVM version.
    /// This may not match the wanted JVM version.
    #[napi(getter)]
    pub fn version(&self) -> napi::Result<String> {
        Ok(self
            .root_vm
            .get_version()
            .map_err(NapiError::to_napi_error)?)
    }

    /// Get the loaded jars.
    #[napi(getter)]
    pub fn loaded_jars(&self) -> &Vec<String> {
        &self.loaded_jars
    }

    /// Append a single or multiple jars to the classpath.
    #[napi(ts_args_type = "classpath: string | string[]")]
    pub fn append_classpath(&mut self, classpath: JsUnknown) -> napi::Result<()> {
        let env = self.root_vm.attach_thread().map_napi_err()?;

        let mut paths = vec![];
        if classpath.is_array()? {
            let obj = classpath.coerce_to_object()?;
            for i in 0..obj.get_array_length()? {
                let path: JsString = obj.get_element(i)?;
                paths.push(path.into_utf16()?.as_str()?);
            }
        } else {
            let path = classpath.coerce_to_string()?;
            paths.push(path.into_utf16()?.as_str()?);
        }

        env.append_class_path(&paths).map_napi_err()?;
        self.loaded_jars.append(&mut paths);

        Ok(())
    }

    /// Set the stdout/stderr callbacks
    #[napi]
    pub fn set_stdout_callbacks(
        &self,
        env: Env,
        #[napi(ts_arg_type = "((err: Error | null, data?: string) => void) | undefined | null")]
        stdout_callback: Option<JsFunction>,
        #[napi(ts_arg_type = "((err: Error | null, data?: string) => void) | undefined | null")]
        stderr_callback: Option<JsFunction>,
    ) -> napi::Result<StdoutRedirect> {
        let j_env = self.root_vm.attach_thread().map_napi_err()?;
        StdoutRedirect::new(
            env,
            &j_env,
            self.root_vm.clone(),
            stdout_callback,
            stderr_callback,
        )
        .map_napi_err()
    }

    #[napi]
    pub fn create_interface_proxy(
        &self,
        env: Env,
        classname: String,
        #[napi(
            ts_arg_type = "Record<string, (err: null | Error, callback: (err: Error | null, data?: any | null) => void, ...args: any[]) => void>"
        )]
        methods: HashMap<String, JsFunction>,
    ) -> napi::Result<JavaInterfaceProxy> {
        JavaInterfaceProxy::new(self.root_vm.clone(), env, classname, methods).map_napi_err()
    }

    pub fn vm(&self) -> JavaVM {
        self.root_vm.clone()
    }
}
