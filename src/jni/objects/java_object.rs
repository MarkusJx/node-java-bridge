use crate::jni::java_type::JavaType;
use crate::jni::objects::class::{GlobalJavaClass, JavaClass};
use crate::jni::objects::object::{GlobalJavaObject, LocalJavaObject};
use crate::jni::objects::string::JavaString;
use crate::jni::objects::value::JavaValue;
use crate::jni::traits::{GetRaw, GetSignature, IsNull, ToJavaValue};
use crate::jni::util::util::ResultType;
use crate::sys;
use std::error::Error;

pub enum JavaObject<'a> {
    LocalRef(&'a LocalJavaObject<'a>),
    Local(LocalJavaObject<'a>),
    Global(GlobalJavaObject),
}

impl<'a> JavaObject<'a> {
    pub fn into_global(self) -> ResultType<GlobalJavaObject> {
        match self {
            Self::LocalRef(_) => Err("Local reference cannot be converted to global".into()),
            Self::Local(local_object) => GlobalJavaObject::try_from(local_object),
            Self::Global(global_object) => Ok(global_object.clone()),
        }
    }

    pub fn clone(&'a self) -> Self {
        match self {
            Self::LocalRef(local_object) => JavaObject::LocalRef(local_object),
            Self::Local(local_object) => JavaObject::LocalRef(&local_object),
            Self::Global(global_object) => JavaObject::Global(global_object.clone()),
        }
    }
}

impl<'a> IsNull for JavaObject<'a> {
    fn is_null(&self) -> bool {
        match self {
            Self::LocalRef(local_object) => local_object.is_null(),
            Self::Local(local_object) => local_object.is_null(),
            Self::Global(global_object) => global_object.is_null(),
        }
    }
}

impl<'a> GetRaw for JavaObject<'a> {
    unsafe fn get_raw_nullable(&self) -> sys::jobject {
        match self {
            Self::LocalRef(local_object) => local_object.get_raw_nullable(),
            Self::Local(local_object) => local_object.get_raw_nullable(),
            Self::Global(global_object) => global_object.get_raw_nullable(),
        }
    }
}

impl TryInto<GlobalJavaObject> for JavaObject<'_> {
    type Error = Box<dyn Error>;

    fn try_into(self) -> ResultType<GlobalJavaObject> {
        self.into_global()
    }
}

impl From<GlobalJavaObject> for JavaObject<'_> {
    fn from(global_java_object: GlobalJavaObject) -> Self {
        Self::Global(global_java_object)
    }
}

impl From<&GlobalJavaObject> for JavaObject<'_> {
    fn from(global_java_object: &GlobalJavaObject) -> Self {
        Self::Global(global_java_object.clone())
    }
}

impl<'a> From<&'a LocalJavaObject<'a>> for JavaObject<'a> {
    fn from(object: &'a LocalJavaObject<'a>) -> Self {
        Self::LocalRef(object)
    }
}

impl<'a> From<LocalJavaObject<'a>> for JavaObject<'a> {
    fn from(object: LocalJavaObject<'a>) -> Self {
        Self::Local(object)
    }
}

impl From<GlobalJavaClass> for JavaObject<'_> {
    fn from(global_java_class: GlobalJavaClass) -> Self {
        Self::Global(global_java_class.to_object())
    }
}

impl From<&GlobalJavaClass> for JavaObject<'_> {
    fn from(global_java_class: &GlobalJavaClass) -> Self {
        Self::Global(global_java_class.clone().to_object())
    }
}

impl<'a> From<&'a JavaString<'a>> for JavaObject<'a> {
    fn from(java_string: &'a JavaString<'a>) -> Self {
        Self::LocalRef(java_string.to_object())
    }
}

impl<'a> From<JavaString<'a>> for JavaObject<'a> {
    fn from(java_string: JavaString<'a>) -> Self {
        Self::Local(java_string.into())
    }
}

impl<'a> From<&'a JavaClass<'a>> for JavaObject<'a> {
    fn from(java_class: &'a JavaClass<'a>) -> Self {
        Self::LocalRef(java_class.to_object())
    }
}

impl<'a> ToJavaValue<'a> for JavaObject<'a> {
    fn to_java_value(&'a self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue {
            l: unsafe { self.get_raw_nullable() },
        })
    }
}

impl<'a> GetSignature for JavaObject<'a> {
    fn get_signature(&self) -> ResultType<JavaType> {
        match self {
            Self::LocalRef(local_object) => local_object.get_signature(),
            Self::Local(local_object) => local_object.get_signature(),
            Self::Global(global_object) => global_object.get_signature(),
        }
    }
}
