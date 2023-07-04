use crate::java::traits::ToJavaValue;
use crate::java_type::Type;
use crate::{define_java_value, sys};
use std::marker::PhantomData;
use std::ptr;

pub struct JavaValue<'a> {
    value: sys::jvalue,
    _marker: PhantomData<&'a sys::jobject>,
}

impl<'a> JavaValue<'a> {
    pub(in crate::java) fn new(value: sys::jvalue) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    pub(in crate::java) unsafe fn value(&self) -> sys::jvalue {
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

    fn get_type(&self) -> Type {
        Type::Object
    }
}

impl<'a> Into<JavaValue<'a>> for JavaNull {
    fn into(self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue { l: ptr::null_mut() })
    }
}

define_java_value!(JavaBoolean, bool, z, Type::Boolean);
define_java_value!(JavaByte, i8, b, Type::Byte);
define_java_value!(JavaInt, i32, i, Type::Integer);
define_java_value!(JavaLong, i64, j, Type::Long);
define_java_value!(JavaFloat, f32, f, Type::Float);
define_java_value!(JavaDouble, f64, d, Type::Double);
define_java_value!(JavaChar, u16, c, Type::Character);
define_java_value!(JavaShort, i16, s, Type::Short);
