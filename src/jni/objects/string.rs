use crate::jni::java_env::JavaEnv;
use crate::jni::java_env_wrapper::JavaEnvWrapper;
use crate::jni::java_type::JavaType;
use crate::jni::objects::object::{GlobalJavaObject, LocalJavaObject};
use crate::jni::objects::value::JavaValue;
use crate::jni::traits::{GetRaw, GetSignature, ToJavaValue};
use crate::jni::util::util::ResultType;
use crate::sys;
use std::error::Error;

pub struct JavaString<'a>(pub(in crate::jni::objects) LocalJavaObject<'a>);

impl<'a> JavaString<'a> {
    pub(in crate::jni) fn new(env: &'a JavaEnvWrapper<'a>, string: sys::jstring) -> Self {
        Self(LocalJavaObject::new(string, env))
    }

    pub(in crate::jni) fn _try_from(
        string: String,
        env: &'a JavaEnvWrapper<'a>,
    ) -> ResultType<Self> {
        env.string_to_java_string(string)
    }

    pub fn try_from(string: String, env: &'a JavaEnv<'a>) -> ResultType<Self> {
        env.get_env().string_to_java_string(string)
    }

    pub fn to_object(&'a self) -> &'a LocalJavaObject<'a> {
        &self.0
    }

    pub fn to_string(self) -> ResultType<String> {
        self.try_into()
    }

    pub fn from_global(object: &'a GlobalJavaObject, env: &'a JavaEnv<'a>) -> Self {
        Self(LocalJavaObject::from(object, env))
    }

    pub unsafe fn from_raw(env: &'a JavaEnv<'a>, string: sys::jstring) -> Self {
        Self(LocalJavaObject::new(string, env.get_env()))
    }
}

impl<'a> GetSignature for JavaString<'a> {
    fn get_signature(&self) -> ResultType<JavaType> {
        self.0.get_signature()
    }
}

impl<'a> ToJavaValue<'a> for JavaString<'a> {
    fn to_java_value(&'a self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue {
            l: unsafe { self.0.get_raw() },
        })
    }
}

impl<'a> Into<LocalJavaObject<'a>> for JavaString<'a> {
    fn into(self) -> LocalJavaObject<'a> {
        self.0
    }
}

impl<'a> From<LocalJavaObject<'a>> for JavaString<'a> {
    fn from(object: LocalJavaObject<'a>) -> Self {
        JavaString(object)
    }
}

impl TryInto<String> for JavaString<'_> {
    type Error = Box<dyn Error>;

    fn try_into(self) -> Result<String, Self::Error> {
        unsafe {
            self.0.env().get_string_utf_chars(
                self.0.get_raw(),
            )
        }
    }
}
