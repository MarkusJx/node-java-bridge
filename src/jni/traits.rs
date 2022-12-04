use crate::jni::java_type::JavaType;
use crate::jni::objects::class::JavaClass;
use crate::jni::objects::value::JavaValue;
use crate::jni::util::util::ResultType;
use crate::sys;

/// Get raw jni pointers from a java object.
pub trait GetRaw {
    /// Get the raw jni pointer from a java object.
    /// The pointer returned must not be null.
    unsafe fn get_raw(&self) -> sys::jobject;
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
