use crate::debug;
use crate::node::class_cache::ClassCache;
use crate::node::config::ClassConfiguration;
use crate::node::helpers::napi_error::{MapToNapiError, StrIntoNapiError};
use crate::node::interface_proxy::interface_proxy_options::InterfaceProxyOptions;
use crate::node::interface_proxy::java_interface_proxy::JavaInterfaceProxy;
use crate::node::java_class_instance::{JavaClassInstance, CLASS_PROXY_PROPERTY, OBJECT_PROPERTY};
use crate::node::java_class_proxy::JavaClassProxy;
use crate::node::java_options::JavaOptions;
use crate::node::stdout_redirect::StdoutRedirect;
use crate::node::util::util::{list_files, parse_array_or_string, parse_classpath_args};
use app_state::{AppStateTrait, MutAppState};
use futures::future;
use java_rs::java_env::JavaEnv;
use java_rs::java_vm::JavaVM;
use java_rs::objects::args::AsJavaArg;
use java_rs::objects::class::JavaClass;
use java_rs::objects::java_object::JavaObject;
use java_rs::objects::object::GlobalJavaObject;
use java_rs::objects::string::JavaString;
use napi::{Env, JsFunction, JsObject, JsUnknown, ValueType};
use std::collections::HashMap;
use std::sync::Arc;

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
        let mut args = opts.unwrap_or(vec![]);
        let mut loaded_jars = vec![];

        debug!("Loading JVM version {}", ver);

        if let Some(cp) = java_options.as_ref().and_then(|o| o.classpath.as_ref()) {
            let cp = list_files(
                cp.clone(),
                java_options
                    .as_ref()
                    .and_then(|o| o.ignore_unreadable_class_path_entries)
                    .unwrap_or(false),
            )?;

            loaded_jars.extend(cp.clone());
            let parsed = parse_classpath_args(&cp, &mut args);
            args.push(parsed);
        }

        debug!("Parsed classpath args: {:?}", args);
        let root_vm = JavaVM::new(&ver, lib_path, &args).map_napi_err()?;

        let env = root_vm.attach_thread().map_napi_err()?;
        env.append_class_path(vec![java_lib_path]).map_napi_err()?;
        let native_library_class =
            JavaClass::by_java_name("io.github.markusjx.bridge.NativeLibrary".to_string(), &env)
                .map_napi_err()?;

        let load_library = native_library_class
            .get_static_void_method("loadLibrary", "(Ljava/lang/String;)V")
            .map_napi_err()?;
        load_library
            .call(&[JavaString::from_string(native_lib_path, &env)
                .map_napi_err()?
                .as_arg()])
            .map_napi_err()?;

        Ok(Self {
            root_vm,
            wanted_version: ver,
            loaded_jars,
        })
    }

    /// Import a java class
    /// Will import the class and parse all of its methods and fields.
    /// The imported class will be cached for future use.
    #[napi(ts_return_type = "object")]
    pub fn import_class(
        &mut self,
        env: Env,
        class_name: String,
        config: Option<ClassConfiguration>,
    ) -> napi::Result<JsFunction> {
        let proxy = MutAppState::<ClassCache>::get_or_insert_default()
            .get_mut()
            .get_class_proxy(&self.root_vm, class_name, config)
            .map_napi_err()?;
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
        config: Option<ClassConfiguration>,
    ) -> napi::Result<JsObject> {
        env.execute_tokio_future(
            future::lazy(|_| {
                MutAppState::<ClassCache>::get_or_insert_default()
                    .get_mut()
                    .get_class_proxy(&self.root_vm, class_name, config)
                    .map_napi_err()
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
        self.root_vm.get_version().map_napi_err()
    }

    /// Get the loaded jars.
    #[napi(getter)]
    pub fn loaded_jars(&self) -> &Vec<String> {
        &self.loaded_jars
    }

    /// Append a single or multiple jars to the classpath.
    #[napi(ts_args_type = "classpath: string | string[]")]
    pub fn append_classpath(
        &mut self,
        classpath: JsUnknown,
        ignore_unreadable: Option<bool>,
    ) -> napi::Result<()> {
        let mut paths = list_files(
            parse_array_or_string(classpath)?,
            ignore_unreadable.unwrap_or(false),
        )?;

        let env = self.root_vm.attach_thread().map_napi_err()?;
        env.append_class_path(paths.clone()).map_napi_err()?;
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
        options: Option<InterfaceProxyOptions>,
    ) -> napi::Result<JavaInterfaceProxy> {
        JavaInterfaceProxy::new(
            self.root_vm.clone(),
            env,
            classname,
            methods,
            options.unwrap_or(InterfaceProxyOptions::default()),
        )
        .map_napi_err()
    }

    /// Check if `this` is instance of `other`
    #[napi]
    pub fn is_instance_of(
        &self,
        node_env: Env,
        this_obj: JsObject,
        #[napi(ts_arg_type = "string | object")] other: JsUnknown,
    ) -> napi::Result<bool> {
        let env = self.root_vm.attach_thread().map_napi_err()?;
        Self::_is_instance_of(env, &node_env, this_obj, other)
    }

    #[napi(getter, ts_return_type = "object")]
    pub fn get_class_loader(&self, env: Env) -> napi::Result<JsUnknown> {
        let proxy = MutAppState::<ClassCache>::get_or_insert_default()
            .get_mut()
            .get_class_proxy(&self.root_vm, "java.net.URLClassLoader".into(), None)
            .map_napi_err()?;
        let j_env = self.root_vm.attach_thread().map_napi_err()?;
        JavaClassInstance::from_existing(proxy, &env, j_env.get_class_loader().map_napi_err()?)
    }

    #[napi(setter)]
    pub fn set_class_loader(
        &self,
        env: Env,
        #[napi(ts_arg_type = "object")] class_loader: JsUnknown,
    ) -> napi::Result<()> {
        let j_env = self.root_vm.attach_thread().map_napi_err()?;
        let obj = class_loader.coerce_to_object()?;
        let instance =
            env.unwrap::<GlobalJavaObject>(&obj.get_named_property::<JsObject>(OBJECT_PROPERTY)?)?;

        j_env.replace_class_loader(instance.clone()).map_napi_err()
    }

    pub fn vm(&self) -> JavaVM {
        self.root_vm.clone()
    }

    pub fn _is_instance_of(
        env: JavaEnv,
        node_env: &Env,
        this: JsObject,
        other: JsUnknown,
    ) -> napi::Result<bool> {
        let other = if other.get_type()? == ValueType::String {
            env.find_global_class_by_java_name(other.coerce_to_string()?.into_utf16()?.as_str()?)
                .map_napi_err()?
        } else if other.get_type()? == ValueType::Function || other.get_type()? == ValueType::Object
        {
            let err_fn = |_| "'other' is not a java object".into_napi_err();
            let obj: JsObject = other
                .coerce_to_object()?
                .get_named_property(CLASS_PROXY_PROPERTY)
                .map_err(err_fn)?;
            node_env
                .unwrap::<Arc<JavaClassProxy>>(&obj)
                .map_err(err_fn)?
                .class
                .clone()
        } else {
            return Err("'other' must be either a string or a java object".into_napi_err());
        };

        let err_fn = |_| "'this' is not a java object".into_napi_err();
        let this_obj: JsObject = this.get_named_property(OBJECT_PROPERTY).map_err(err_fn)?;
        let this = node_env
            .unwrap::<GlobalJavaObject>(&this_obj)
            .map_err(err_fn)?;

        env.instance_of(JavaObject::from(this.clone()), other)
            .map_napi_err()
    }
}

/// Clear the class proxy cache.
/// Use this method in order to reset the config for all class proxies.
/// The new config will be applied once the class is imported again.
///
/// @since 2.4.0
#[napi]
pub fn clear_class_proxies() {
    MutAppState::<ClassCache>::get_or_insert_default()
        .get_mut()
        .clear();
}
