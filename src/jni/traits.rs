use crate::jni::java_type::JavaType;
use crate::jni::objects::class::JavaClass;
use crate::jni::objects::value::JavaValue;
use crate::jni::util::util::ResultType;
use crate::sys;
use std::ptr;

pub trait GetRaw {
    unsafe fn get_raw_nullable(&self) -> sys::jobject;

    unsafe fn get_raw(&self) -> Option<sys::jobject> {
        let raw = self.get_raw_nullable();
        if raw == ptr::null_mut() {
            None
        } else {
            Some(raw)
        }
    }
}

pub trait IsNull {
    fn is_null(&self) -> bool;
}

pub trait ToJavaValue<'a> {
    fn to_java_value(&'a self) -> JavaValue<'a>;
}

pub trait GetSignature {
    fn get_signature(&self) -> ResultType<JavaType>;
}

pub trait IsInstanceOf {
    fn is_instance_of(&self, classname: &str) -> ResultType<bool>;
}

pub trait GetClass {
    fn get_class(&self) -> ResultType<JavaClass>;
}
