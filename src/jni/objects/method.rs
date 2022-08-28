use crate::jni::java_env::JavaEnv;
use crate::jni::java_type::{JavaType, Type};
use crate::jni::objects::args::JavaArgs;
use crate::jni::objects::class::{GlobalJavaClass, JavaClass};
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::object::LocalJavaObject;
use crate::jni::util::util::ResultType;
use crate::{define_java_methods, sys};
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct JavaMethod<'a> {
    method: sys::jmethodID,
    class: &'a JavaClass<'a>,
    return_type: JavaType,
    is_static: bool,
}

impl<'a> JavaMethod<'a> {
    pub(in crate::jni) fn new(
        method: sys::jmethodID,
        class: &'a JavaClass<'a>,
        return_type: JavaType,
        is_static: bool,
    ) -> Self {
        Self {
            method,
            class,
            return_type,
            is_static,
        }
    }

    pub(in crate::jni) unsafe fn id(&'a self) -> sys::jmethodID {
        self.method
    }
}

pub struct GlobalJavaMethod {
    method: AtomicPtr<sys::_jmethodID>,
    class: GlobalJavaClass,
    return_type: JavaType,
    is_static: bool,
}

impl GlobalJavaMethod {
    pub fn from(class: GlobalJavaClass, method: JavaMethod) -> Self {
        Self {
            method: AtomicPtr::new(method.method),
            class,
            return_type: method.return_type,
            is_static: method.is_static,
        }
    }

    pub fn get_class<'a, 'b>(&'a self, env: &'b JavaEnv<'b>) -> JavaClass<'b>
    where
        'a: 'b,
    {
        JavaClass::from_global(&self.class, env)
    }
}

impl Clone for GlobalJavaMethod {
    fn clone(&self) -> Self {
        Self {
            method: AtomicPtr::new(self.method.load(Ordering::Relaxed)),
            class: self.class.clone(),
            return_type: self.return_type.clone(),
            is_static: self.is_static,
        }
    }
}

define_java_methods!(
    JavaObjectMethod,
    BoundJavaObjectMethod,
    StaticJavaObjectMethod,
    call_object_method,
    call_static_object_method,
    LocalJavaObject<'_>,
    [
        Type::Object,
        Type::LangBoolean,
        Type::String,
        Type::LangCharacter,
        Type::LangByte,
        Type::LangFloat,
        Type::LangDouble,
        Type::LangInteger,
        Type::LangLong,
        Type::LangObject,
        Type::LangShort,
        Type::Array
    ]
);

// Additional methods for JavaObjectMethod
impl<'a> JavaObjectMethod<'a> {
    pub fn call_with_errors(
        &'a self,
        object: JavaObject<'a>,
        args: JavaArgs<'a>,
        resolve_errors: bool,
    ) -> ResultType<LocalJavaObject<'a>> {
        self.0
            .class
            .env()
            .call_object_method_with_errors(object, &self.0, args, resolve_errors)
    }
}

define_java_methods!(
    JavaIntMethod,
    BoundJavaIntMethod,
    StaticJavaIntMethod,
    call_int_method,
    call_static_int_method,
    i32,
    [Type::Integer]
);
define_java_methods!(
    JavaBooleanMethod,
    BoundJavaBooleanMethod,
    StaticJavaBooleanMethod,
    call_boolean_method,
    call_static_boolean_method,
    bool,
    [Type::Boolean]
);
define_java_methods!(
    JavaVoidMethod,
    BoundJavaVoidMethod,
    StaticJavaVoidMethod,
    call_void_method,
    call_static_void_method,
    (),
    [Type::Void]
);
define_java_methods!(
    JavaByteMethod,
    BoundJavaByteMethod,
    StaticJavaByteMethod,
    call_byte_method,
    call_static_byte_method,
    i8,
    [Type::Byte]
);
define_java_methods!(
    JavaCharMethod,
    BoundJavaCharMethod,
    StaticJavaCharMethod,
    call_char_method,
    call_static_char_method,
    u16,
    [Type::Character]
);
define_java_methods!(
    JavaShortMethod,
    BoundJavaShortMethod,
    StaticJavaShortMethod,
    call_short_method,
    call_static_short_method,
    i16,
    [Type::Short]
);
define_java_methods!(
    JavaLongMethod,
    BoundJavaLongMethod,
    StaticJavaLongMethod,
    call_long_method,
    call_static_long_method,
    i64,
    [Type::Long]
);
define_java_methods!(
    JavaFloatMethod,
    BoundJavaFloatMethod,
    StaticJavaFloatMethod,
    call_float_method,
    call_static_float_method,
    f32,
    [Type::Float]
);
define_java_methods!(
    JavaDoubleMethod,
    BoundJavaDoubleMethod,
    StaticJavaDoubleMethod,
    call_double_method,
    call_static_double_method,
    f64,
    [Type::Double]
);
