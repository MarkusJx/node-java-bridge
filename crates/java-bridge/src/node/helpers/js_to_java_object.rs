use crate::node::java_class_instance::{CLASS_PROXY_PROPERTY, OBJECT_PROPERTY};
use crate::node::java_class_proxy::JavaClassProxy;
use java_rs::objects::object::GlobalJavaObject;
use napi::{Env, JsObject, JsUnknown};
use std::sync::Arc;

pub trait JsIntoJavaObject {
    fn into_java_object(self, env: &Env) -> napi::Result<GlobalJavaObject>;
}

pub trait JsToJavaClass {
    fn to_java_class(&self, env: &Env) -> napi::Result<Arc<JavaClassProxy>>;
}

impl JsIntoJavaObject for JsUnknown {
    fn into_java_object(self, env: &Env) -> napi::Result<GlobalJavaObject> {
        let obj: JsObject = self
            .coerce_to_object()?
            .get_named_property(OBJECT_PROPERTY)?;
        Ok(env.unwrap::<GlobalJavaObject>(&obj)?.clone())
    }
}

impl JsToJavaClass for JsObject {
    fn to_java_class(&self, env: &Env) -> napi::Result<Arc<JavaClassProxy>> {
        let obj: JsObject = self.get_named_property(CLASS_PROXY_PROPERTY)?;
        Ok(env.unwrap::<Arc<JavaClassProxy>>(&obj)?.clone())
    }
}
