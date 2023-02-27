use crate::node::interface_proxy::interface_call::InterfaceCall;
use crate::node::interface_proxy::js_error::JsError;
use java_rs::objects::object::GlobalJavaObject;
use napi::threadsafe_function::ThreadsafeFunction;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type MethodMap = HashMap<String, ThreadsafeFunction<InterfaceCall>>;
pub type MethodsType = Arc<Mutex<MethodMap>>;
pub type ProxiesType = HashMap<usize, MethodsType>;
pub type JsCallResult = Result<Result<Option<GlobalJavaObject>, JsError>, String>;
