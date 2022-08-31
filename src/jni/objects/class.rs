use crate::jni::java_env::JavaEnv;
use crate::jni::java_env_wrapper::JavaEnvWrapper;
use crate::jni::java_field::JavaField;
use crate::jni::java_type::JavaType;
use crate::jni::objects::constructor::JavaConstructor;
use crate::jni::objects::method::{
    JavaBooleanMethod, JavaByteMethod, JavaCharMethod, JavaDoubleMethod, JavaFloatMethod,
    JavaIntMethod, JavaLongMethod, JavaObjectMethod, JavaShortMethod, JavaVoidMethod,
    StaticJavaBooleanMethod, StaticJavaByteMethod, StaticJavaCharMethod, StaticJavaDoubleMethod,
    StaticJavaFloatMethod, StaticJavaIntMethod, StaticJavaLongMethod, StaticJavaObjectMethod,
    StaticJavaShortMethod, StaticJavaVoidMethod,
};
use crate::jni::objects::object::{GlobalJavaObject, LocalJavaObject};
use crate::jni::traits::GetRaw;
use crate::jni::util::util::ResultType;
use crate::{define_get_method_method, sys};
use std::error::Error;

pub struct JavaClass<'a>(LocalJavaObject<'a>);

impl<'a> JavaClass<'a> {
    pub fn new(object: sys::jclass, env: &'a JavaEnvWrapper<'a>) -> Self {
        Self(LocalJavaObject::new(object, env))
    }

    pub fn by_name(name: &str, env: &'a JavaEnv<'a>) -> ResultType<Self> {
        env.find_class(name)
    }

    pub fn by_java_name(name: String, env: &'a JavaEnv<'a>) -> ResultType<Self> {
        env.find_class_by_java_name(name)
    }

    pub fn get_object_method(
        &'a self,
        method_name: &str,
        signature: &str,
    ) -> ResultType<JavaObjectMethod<'a>> {
        self.get_object_method_with_errors(method_name, signature, true)
    }

    pub(in crate::jni) fn env(&'a self) -> &'a JavaEnvWrapper<'a> {
        self.0.env()
    }

    define_get_method_method!(get_int_method, get_method_id, JavaIntMethod);
    define_get_method_method!(get_long_method, get_method_id, JavaLongMethod);
    define_get_method_method!(get_double_method, get_method_id, JavaDoubleMethod);
    define_get_method_method!(get_float_method, get_method_id, JavaFloatMethod);
    define_get_method_method!(get_boolean_method, get_method_id, JavaBooleanMethod);
    define_get_method_method!(get_short_method, get_method_id, JavaShortMethod);
    define_get_method_method!(get_byte_method, get_method_id, JavaByteMethod);
    define_get_method_method!(get_char_method, get_method_id, JavaCharMethod);
    define_get_method_method!(get_void_method, get_method_id, JavaVoidMethod);

    define_get_method_method!(
        get_static_int_method,
        get_static_method_id,
        StaticJavaIntMethod
    );
    define_get_method_method!(
        get_static_long_method,
        get_static_method_id,
        StaticJavaLongMethod
    );
    define_get_method_method!(
        get_static_double_method,
        get_static_method_id,
        StaticJavaDoubleMethod
    );
    define_get_method_method!(
        get_static_float_method,
        get_static_method_id,
        StaticJavaFloatMethod
    );
    define_get_method_method!(
        get_static_short_method,
        get_static_method_id,
        StaticJavaShortMethod
    );
    define_get_method_method!(
        get_static_byte_method,
        get_static_method_id,
        StaticJavaByteMethod
    );
    define_get_method_method!(
        get_static_char_method,
        get_static_method_id,
        StaticJavaCharMethod
    );
    define_get_method_method!(
        get_static_void_method,
        get_static_method_id,
        StaticJavaVoidMethod
    );

    pub fn get_object_method_with_errors(
        &'a self,
        method_name: &str,
        signature: &str,
        resolve_errors: bool,
    ) -> ResultType<JavaObjectMethod<'a>> {
        let method = self.0.env().get_method_id_with_errors(
            &self,
            method_name,
            signature,
            resolve_errors,
        )?;

        Ok(JavaObjectMethod::new(method))
    }

    pub fn get_static_object_method(
        &'a self,
        method_name: &str,
        signature: &str,
    ) -> ResultType<StaticJavaObjectMethod<'a>> {
        let method = self
            .0
            .env()
            .get_static_method_id(&self, method_name, signature)?;

        Ok(StaticJavaObjectMethod::new(method))
    }

    pub fn get_static_boolean_method(
        &'a self,
        method_name: &str,
        signature: &str,
    ) -> ResultType<StaticJavaBooleanMethod<'a>> {
        let method = self
            .0
            .env()
            .get_static_method_id(&self, method_name, signature)?;

        Ok(StaticJavaBooleanMethod::new(method))
    }

    pub fn get_constructor(&self, signature: &str) -> ResultType<JavaConstructor> {
        self.env().get_constructor(self, signature)
    }

    pub fn get_field(
        &'a self,
        name: String,
        signature: JavaType,
        is_static: bool,
    ) -> ResultType<JavaField<'a>> {
        self.0.env().get_field_id(&self, name, signature, is_static)
    }

    pub fn from_global(object: &'a GlobalJavaClass, env: &'a JavaEnv<'a>) -> Self {
        Self(LocalJavaObject::from(&object.0, env))
    }

    pub fn to_object(&'a self) -> &'a LocalJavaObject<'a> {
        &self.0
    }

    pub fn is_assignable_from(&self, other: &JavaClass) -> ResultType<bool> {
        unsafe {
            self.0
                .env()
                .is_assignable_from(other.class()?, self.class()?)
        }
    }

    pub(in crate::jni) unsafe fn class(&self) -> ResultType<sys::jclass> {
        self.0
            .get_raw()
            .ok_or("Cannot get class of null pointer".into())
    }
}

impl<'a> From<LocalJavaObject<'a>> for JavaClass<'a> {
    fn from(object: LocalJavaObject<'a>) -> Self {
        Self(object)
    }
}

#[derive(Clone)]
pub struct GlobalJavaClass(GlobalJavaObject);

impl GlobalJavaClass {
    pub fn by_name(name: &str, env: &JavaEnv<'_>) -> ResultType<Self> {
        env.find_global_class_by_java_name(name.replace('/', "."))
    }

    pub fn to_object(self) -> GlobalJavaObject {
        self.0
    }

    pub(in crate::jni) unsafe fn class(&self) -> ResultType<sys::jclass> {
        self.0
            .get_raw()
            .ok_or("Cannot get class of null pointer".into())
    }
}

impl From<GlobalJavaObject> for GlobalJavaClass {
    fn from(object: GlobalJavaObject) -> Self {
        Self(object)
    }
}

impl TryFrom<JavaClass<'_>> for GlobalJavaClass {
    type Error = Box<dyn Error>;

    fn try_from(class: JavaClass) -> ResultType<Self> {
        let global = GlobalJavaObject::try_from(class.0)?;
        Ok(Self(global))
    }
}

impl<'a> TryFrom<LocalJavaObject<'a>> for GlobalJavaClass {
    type Error = Box<dyn Error>;

    fn try_from(object: LocalJavaObject<'a>) -> ResultType<Self> {
        let global = GlobalJavaObject::try_from(object)?;
        Ok(Self(global))
    }
}
