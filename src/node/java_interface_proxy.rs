use crate::node::extensions::java_call_result_ext::ToNapiValue;
use crate::node::extensions::java_type_ext::NapiToJava;
use crate::node::helpers::napi_error::MapToNapiError;
use futures::channel::oneshot::{channel, Sender};
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
use lazy_static::lazy_static;
use napi::threadsafe_function::{
    ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use napi::{CallContext, Env, JsFunction, JsObject, JsString, JsUnknown, Status};
use rand::Rng;
use std::collections::HashMap;
use std::ptr;
use std::sync::{Arc, Mutex, MutexGuard};

type MethodsType = Arc<Mutex<HashMap<String, ThreadsafeFunction<InterfaceCall>>>>;
type ProxiesType = HashMap<usize, MethodsType>;
type JsCallResult = Result<Result<Option<GlobalJavaObject>, JsError>, String>;

lazy_static! {
    static ref PROXIES: Mutex<ProxiesType> = Mutex::new(HashMap::new());
}

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
                    env.throw_error(err.message);
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

    let proxies = PROXIES.lock().unwrap();
    let methods = proxies
        .get(&(id as _))
        .ok_or(format!("No proxy with the id {} exists", id))?
        .lock()
        .unwrap();
    let method = methods
        .get(&name)
        .ok_or(format!("No method with the name {} exists", name))?;

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

    let res = futures::executor::block_on(rx)??;
    Ok(res.map(|o| o.map(|g| g.into_return_value()).unwrap_or(ptr::null_mut())))
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

        JsError::_push(&mut stack, function!(), file!(), line!());
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

struct JsError {
    message: String,
    stack: Vec<String>,
}

impl JsError {
    pub fn new(message: String, mut stack: Vec<String>) -> Self {
        Self::_push(&mut stack, function!(), file!(), line!());
        Self { message, stack }
    }

    pub fn push(&mut self, method: &str, file: &str, line: u32) {
        Self::_push(&mut self.stack, method, file, line);
    }

    fn _push(stack: &mut Vec<String>, method: &str, file: &str, line: u32) {
        stack.insert(0, format!("\tat {} ({}:{})", method, file, line));
    }

    fn throw(&self, env: &JavaEnv) -> ResultType<()> {
        let utils = JavaClass::by_name("io/github/markusjx/bridge/Util", &env)?;
        let exception_from_js_error = utils.get_static_object_method(
            "exceptionFromJsError",
            "(Ljava/lang/String;[Ljava/lang/String;)Ljava/lang/Exception;",
        )?;

        let mut stack = self.stack.clone();
        Self::_push(&mut stack, function!(), file!(), line!());

        let string_class = JavaClass::by_name("java/lang/String", &env)?;
        let mut java_stack = JavaObjectArray::new(&string_class, stack.len() as _)?;

        for i in 0..stack.len() {
            java_stack.set(
                i as _,
                Some(JavaObject::from(JavaString::from_string(
                    stack.get(i).unwrap().clone(),
                    &env,
                )?)),
            )?;
        }

        let exception = exception_from_js_error
            .call(&[
                JavaString::from_string(self.message.clone(), &env)?.as_arg(),
                java_stack.as_arg(),
            ])?
            .ok_or(
                "io/github/markusjx/bridge/Util.exceptionFromJsError returned null".to_string(),
            )?;
        env.throw(JavaObject::from(exception));
        Ok(())
    }
}

struct InterfaceCall {
    args: Vec<JavaCallResult>,
    sender: Mutex<Option<Sender<JsCallResult>>>,
}

impl InterfaceCall {
    pub fn new(args: Vec<JavaCallResult>, sender: Sender<JsCallResult>) -> Self {
        InterfaceCall {
            args,
            sender: Mutex::new(Some(sender)),
        }
    }

    pub fn set_result(&self, result: JsCallResult) -> ResultType<()> {
        self.sender
            .lock()
            .unwrap()
            .take()
            .ok_or("The sender was already invoked".to_string())?
            .send(result)
            .map_err(|_| "Could not send result to sender".into())
    }
}

#[napi]
pub struct JavaInterfaceProxy {
    id: usize,
    methods: MethodsType,
    proxy_instance: Option<GlobalJavaObject>,
    function_caller_instance: Option<GlobalJavaObject>,
    vm: JavaVM,
}

#[napi]
impl JavaInterfaceProxy {
    pub fn new(
        vm: JavaVM,
        env: Env,
        classname: String,
        methods: HashMap<String, JsFunction>,
    ) -> ResultType<Self> {
        let j_env = vm.attach_thread()?;

        let mut proxies = PROXIES.lock().unwrap();
        let id = Self::generate_id(&proxies);

        let string = JavaClass::by_name("java/lang/String", &j_env)?;
        let mut implemented_methods = JavaObjectArray::new(&string, methods.len())?;
        for i in 0..methods.len() {
            let str = JavaString::from_string(methods.keys().nth(i).unwrap().into(), &j_env)?;
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
            function_caller_instance: Some(global_function_caller_instance),
            proxy_instance: Some(global_proxy_instance),
            vm,
        })
    }

    fn generate_id(proxies: &MutexGuard<ProxiesType>) -> usize {
        let mut rng = rand::thread_rng();
        let mut id: usize = rng.gen();

        while proxies.contains_key(&id) {
            id = rng.gen();
        }

        id
    }

    #[napi(getter)]
    pub fn proxy(&self, env: Env) -> napi::Result<JsObject> {
        let mut res = env.create_object()?;
        env.wrap(&mut res, self.proxy_instance.as_ref().unwrap().clone())?;

        Ok(res)
    }

    #[napi]
    pub fn reset(&mut self) -> napi::Result<()> {
        let mut methods = self.methods.lock().unwrap();
        if self.function_caller_instance.is_none() || self.proxy_instance.is_none() {
            return Err(napi::Error::new(
                Status::Unknown,
                "This instance is already destroyed".into(),
            ));
        }

        let env = self.vm.attach_thread().map_napi_err()?;
        let java_class = env
            .get_object_class(JavaObject::from(
                self.function_caller_instance.as_ref().unwrap(),
            ))
            .map_napi_err()?;
        let destruct = java_class
            .get_void_method("destruct", "()V")
            .map_napi_err()?;
        destruct
            .call(
                JavaObject::from(self.function_caller_instance.as_ref().unwrap()),
                &[],
            )
            .map_napi_err()?;

        let mut proxies = PROXIES.lock().unwrap();
        proxies.remove(&self.id);

        self.proxy_instance.take();
        self.function_caller_instance.take();
        methods.clear();

        Ok(())
    }
}

impl Drop for JavaInterfaceProxy {
    fn drop(&mut self) {
        self.reset().ok();
    }
}
