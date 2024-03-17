use crate::node::extensions::java_call_result_ext::ToNapiValue;
use crate::node::extensions::java_type_ext::NapiToJava;
use crate::node::helpers::napi_error::{MapToNapiError, NapiError};
use crate::node::interface_proxy::function_caller::FunctionCaller;
use crate::node::interface_proxy::interface_call::InterfaceCall;
use crate::node::interface_proxy::interface_proxy_options::InterfaceProxyOptions;
use crate::node::interface_proxy::js_error::JsError;
use crate::node::interface_proxy::proxies::{
    find_methods_by_id, generate_proxy_id, get_daemon_proxies, get_proxies, remove_proxy,
};
use crate::node::interface_proxy::types::{JsCallResult, MethodsType};
use futures::channel::oneshot::channel;
use java_rs::java_call_result::JavaCallResult;
use java_rs::java_env::JavaEnv;
use java_rs::java_type::JavaType;
use java_rs::java_vm::JavaVM;
use java_rs::objects::args::AsJavaArg;
use java_rs::objects::array::JavaObjectArray;
use java_rs::objects::class::JavaClass;
use java_rs::objects::java_object::JavaObject;
use java_rs::objects::object::{GlobalJavaObject, LocalJavaObject};
use java_rs::objects::string::JavaString;
use java_rs::objects::value::JavaLong;
use java_rs::util::util::ResultType;
use java_rs::{function, sys};
use napi::threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunctionCallMode};
use napi::{CallContext, Env, JsFunction, JsObject, JsString, JsUnknown};
use std::collections::HashMap;
use std::ptr;
use std::sync::{Arc, Mutex};

#[no_mangle]
#[allow(non_snake_case, dead_code)]
pub extern "system" fn Java_io_github_markusjx_bridge_JavaFunctionCaller_callNodeFunction(
    env: *mut sys::JNIEnv,
    _: sys::jobject,
    id: sys::jlong,
    method: sys::jobject,
    args: sys::jobjectArray,
) -> sys::jobject {
    let res = unsafe { call_node_function(env, id, method, args) };
    match res {
        Ok(obj) => match obj {
            Ok(obj) => obj,
            Err(mut err) => {
                let env = unsafe { JavaEnv::from_raw(env) };
                err.push(function!(), file!(), line!());
                if err.throw(&env).is_err() {
                    env.throw_error(err.message());
                }

                ptr::null_mut()
            }
        },
        Err(err) => {
            let err_str = err.to_string();
            let env = unsafe { JavaEnv::from_raw(env) };

            env.throw_error(err_str);
            ptr::null_mut()
        }
    }
}

unsafe fn call_node_function(
    env: *mut sys::JNIEnv,
    id: sys::jlong,
    method: sys::jobject,
    args: sys::jobjectArray,
) -> ResultType<Result<sys::jobject, JsError>> {
    let env = JavaEnv::from_raw(env);
    let method = LocalJavaObject::from_raw(method, &env, None);
    let method_class = env.get_object_class(JavaObject::from(&method))?;
    let get_name = method_class.get_object_method("getName", "()Ljava/lang/String;")?;

    let java_name = get_name
        .call(JavaObject::from(&method), &[])?
        .ok_or("Class.getName() returned null")?;
    let name = JavaString::try_from(java_name)?.to_string()?;

    let proxies = get_proxies();
    let daemon_proxies = get_daemon_proxies();
    let methods = find_methods_by_id(id as _, &proxies, &daemon_proxies)?;
    let method = methods
        .get(&name)
        .ok_or(format!("No method with the name '{}' exists", name))?;

    let mut converted_args: Vec<JavaCallResult> = Vec::new();
    if args != ptr::null_mut() {
        let args = JavaObjectArray::from_raw(args, &env, None);
        for i in 0..args.len()? {
            let arg = args.get(i)?;
            converted_args.push(if let Some(arg) = arg {
                JavaCallResult::try_from(JavaObject::from(arg))?
            } else {
                JavaCallResult::Null
            });
        }
    }

    let (tx, rx) = channel::<JsCallResult>();

    method.call(
        Ok(InterfaceCall::new(converted_args, tx)),
        ThreadsafeFunctionCallMode::NonBlocking,
    );
    drop(methods);
    drop(proxies);
    drop(daemon_proxies);

    let res = futures::executor::block_on(rx)??;
    Ok(res.map(|o| {
        o.map(|g| g.into_return_value(&env))
            .unwrap_or(ptr::null_mut())
    }))
}

fn js_callback(
    ctx: &CallContext,
    vm: &JavaVM,
) -> ResultType<Result<Option<GlobalJavaObject>, JsError>> {
    let err = ctx.get::<JsUnknown>(0)?;

    if err.is_error()? {
        let obj = err.coerce_to_object()?;
        let message = obj
            .get_named_property::<JsString>("message")?
            .into_utf16()?
            .as_str()?;

        let mut stack = if obj.has_named_property("stack")? {
            obj.get_named_property::<JsString>("stack")
                .ok()
                .and_then(|s| s.into_utf16().ok())
                .and_then(|s| s.as_str().ok())
                .map(|stack| {
                    stack
                        .split("\n")
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                })
                .map(|mut stack| {
                    stack.remove(0);
                    stack
                })
        } else {
            None
        }
        .unwrap_or(vec![]);

        JsError::push_stack(&mut stack, function!(), file!(), line!());
        Ok(Err(JsError::new(message, stack)))
    } else {
        let env = vm.attach_thread()?;
        let result = ctx.get::<JsUnknown>(1)?;
        let converted = JavaType::object().convert_to_java_object(&env, &ctx.env, result)?;

        Ok(Ok(if let Some(converted) = converted {
            Some(converted.into_global()?)
        } else {
            None
        }))
    }
}

#[napi]
pub struct JavaInterfaceProxy {
    id: usize,
    methods: MethodsType,
    proxy_instance: Option<GlobalJavaObject>,
    function_caller_instance: FunctionCaller,
    options: InterfaceProxyOptions,
}

#[napi]
impl JavaInterfaceProxy {
    pub fn new(
        vm: JavaVM,
        env: Env,
        classname: String,
        methods: HashMap<String, JsFunction>,
        options: InterfaceProxyOptions,
    ) -> ResultType<Self> {
        let j_env = vm.attach_thread()?;

        let mut proxies = get_proxies();
        let daemon_proxies = get_daemon_proxies();
        let id = generate_proxy_id(&proxies, &daemon_proxies);

        let string = JavaClass::by_name("java/lang/String", &j_env)?;
        let mut implemented_methods = JavaObjectArray::new(&string, methods.len())?;
        for i in 0..methods.len() {
            let str = JavaString::from_string(methods.keys().nth(i).unwrap().to_string(), &j_env)?;
            implemented_methods.set(i as _, Some(JavaObject::from(str)))?;
        }

        let java_class = JavaClass::by_java_name(
            "io.github.markusjx.bridge.JavaFunctionCaller".into(),
            &j_env,
        )?;
        let constructor = java_class.get_constructor("([Ljava/lang/String;J)V")?;

        let instance = constructor.new_instance(
            &j_env,
            &[
                implemented_methods.as_arg(),
                JavaLong::new(id as _).as_arg(),
            ],
        )?;

        let proxy = JavaClass::by_name("java/lang/reflect/Proxy", &j_env)?;
        let new_proxy_instance = proxy.get_static_object_method("newProxyInstance", "(Ljava/lang/ClassLoader;[Ljava/lang/Class;Ljava/lang/reflect/InvocationHandler;)Ljava/lang/Object;")?;

        let class = j_env.get_java_lang_class()?;
        let proxied_class = JavaClass::by_java_name(classname, &j_env)?;
        let mut classes = JavaObjectArray::new(&class, 1)?;
        classes.set(0, Some(JavaObject::from(&proxied_class)))?;

        let proxy_instance = new_proxy_instance
            .call(&[
                j_env.get_class_loader()?.as_arg(),
                classes.as_arg(),
                instance.as_arg(),
            ])?
            .ok_or("java.lang.reflect.Proxy.newProxyInstance returned null".to_string())?;

        let global_proxy_instance = GlobalJavaObject::try_from(proxy_instance)?;
        let global_function_caller_instance = GlobalJavaObject::try_from(instance)?;

        let mut converted_methods = HashMap::new();
        for (name, method) in methods.into_iter() {
            let vm_copy = vm.clone();
            converted_methods.insert(
                name,
                env.create_threadsafe_function(
                    &method,
                    0,
                    move |ctx: ThreadSafeCallContext<InterfaceCall>| {
                        let args = ctx.value.args.clone();
                        let callback_vm = vm_copy.clone();
                        let mut res = vec![ctx
                            .env
                            .create_function_from_closure("callback", move |ctx1| {
                                ctx.value
                                    .set_result(
                                        js_callback(&ctx1, &callback_vm).map_err(|e| e.to_string()),
                                    )
                                    .map_napi_err()?;
                                ctx1.env.get_undefined()
                            })?
                            .into_unknown()];
                        for value in args {
                            let env = vm_copy.attach_thread().map_napi_err()?;
                            res.push(value.to_napi_value(&env, &ctx.env).map_napi_err()?);
                        }

                        Ok(res)
                    },
                )?,
            );
        }

        let converted_methods = Arc::new(Mutex::new(converted_methods));
        proxies.insert(id, converted_methods.clone());

        Ok(Self {
            id,
            methods: converted_methods,
            function_caller_instance: FunctionCaller::new(global_function_caller_instance),
            proxy_instance: Some(global_proxy_instance),
            options,
        })
    }

    #[napi(getter)]
    pub fn proxy(&self, env: Env) -> napi::Result<Option<JsObject>> {
        self.proxy_instance.as_ref().map_or(Ok(None), |proxy| {
            let mut res = env.create_object()?;
            env.wrap(&mut res, proxy.clone())?;

            Ok(Some(res))
        })
    }

    #[napi]
    pub fn reset(&mut self, force: Option<bool>) -> napi::Result<()> {
        let mut methods = self.methods.lock().unwrap();
        if self.function_caller_instance.is_dead() || self.proxy_instance.is_none() {
            return Err(NapiError::from("This instance is already destroyed").into());
        }

        let keep_as_daemon =
            self.options.keep_as_daemon.unwrap_or(false) && !force.unwrap_or(false);
        if !keep_as_daemon {
            self.function_caller_instance.destroy()?;
        }

        let mut proxies = get_proxies();
        let mut daemon_proxies = get_daemon_proxies();
        remove_proxy(
            self.id,
            keep_as_daemon,
            &mut proxies,
            &mut daemon_proxies,
            self.function_caller_instance.move_to(),
        );

        self.proxy_instance.take();
        if !keep_as_daemon {
            methods.clear();
        }

        Ok(())
    }
}

impl Drop for JavaInterfaceProxy {
    fn drop(&mut self) {
        self.reset(Some(false)).ok();
    }
}
