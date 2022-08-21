use crate::jni::traits::ToJavaValue;
use crate::{define_java_value, sys};
use std::marker::PhantomData;
use std::ptr;

pub struct JavaValue<'a> {
    value: sys::jvalue,
    _marker: PhantomData<&'a sys::jobject>,
}

impl<'a> JavaValue<'a> {
    pub(in crate::jni) fn new(value: sys::jvalue) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    pub(in crate::jni) unsafe fn value(&self) -> sys::jvalue {
        self.value
    }
}

pub struct JavaNull;

impl JavaNull {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> ToJavaValue<'a> for JavaNull {
    fn to_java_value(&'a self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue { l: ptr::null_mut() })
    }
}

impl<'a> Into<JavaValue<'a>> for JavaNull {
    fn into(self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue { l: ptr::null_mut() })
    }
}

define_java_value!(JavaBoolean, bool, z);
define_java_value!(JavaByte, i8, b);
define_java_value!(JavaInt, i32, i);
define_java_value!(JavaLong, i64, j);
define_java_value!(JavaFloat, f32, f);
define_java_value!(JavaDouble, f64, d);
define_java_value!(JavaChar, u16, c);
define_java_value!(JavaShort, i16, s);
