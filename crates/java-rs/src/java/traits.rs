use crate::java::java_type::JavaType;
use crate::java::objects::class::JavaClass;
use crate::java::objects::value::JavaValue;
use crate::java::util::util::ResultType;
use crate::java_type::Type;
use crate::sys;

/// Get raw jni pointers from a java object.
pub trait GetRaw {
    /// Get the raw jni pointer from a java object.
    /// The pointer returned must not be null.
    unsafe fn get_raw(&self) -> sys::jobject;
}

pub trait ToJavaValue<'a> {
    fn to_java_value(&'a self) -> JavaValue<'a>;
    fn get_type(&self) -> Type;
}

pub trait GetSignature {
    fn get_signature(&self) -> JavaType;
}

pub trait GetSignatureRef {
    fn get_signature_ref(&self) -> &JavaType;
}

pub trait IsInstanceOf {
    fn is_instance_of(&self, classname: &str) -> ResultType<bool>;
}

pub trait GetClass {
    fn get_class(&self) -> ResultType<JavaClass>;
}
