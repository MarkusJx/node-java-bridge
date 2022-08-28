use crate::jni::java_type::JavaType;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::object::GlobalJavaObject;
use crate::jni::objects::value::{
    JavaBoolean, JavaByte, JavaChar, JavaDouble, JavaFloat, JavaInt, JavaLong, JavaNull, JavaShort,
    JavaValue,
};
use crate::jni::traits::{GetSignature, ToJavaValue};
use crate::jni::util::util::ResultType;
use std::error::Error;

#[derive(Clone)]
pub enum JavaCallResult {
    Void,
    Null,
    Boolean(bool),
    Byte(i8),
    Character(u16),
    Short(i16),
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Object {
        object: GlobalJavaObject,
        signature: JavaType,
    },
}

unsafe impl Send for JavaCallResult {}

impl<'a> ToJavaValue<'a> for JavaCallResult {
    fn to_java_value(&'a self) -> JavaValue<'a> {
        match self {
            JavaCallResult::Void | JavaCallResult::Null => JavaNull::new().into(),
            JavaCallResult::Boolean(b) => JavaBoolean::new(*b).into(),
            JavaCallResult::Byte(b) => JavaByte::new(*b).into(),
            JavaCallResult::Character(c) => JavaChar::new(*c).into(),
            JavaCallResult::Short(s) => JavaShort::new(*s).into(),
            JavaCallResult::Integer(i) => JavaInt::new(*i).into(),
            JavaCallResult::Long(l) => JavaLong::new(*l).into(),
            JavaCallResult::Float(f) => JavaFloat::new(*f).into(),
            JavaCallResult::Double(d) => JavaDouble::new(*d).into(),
            JavaCallResult::Object { object, .. } => object.to_java_value(),
        }
    }
}

impl TryFrom<JavaObject<'_>> for JavaCallResult {
    type Error = Box<dyn Error>;

    fn try_from(value: JavaObject) -> ResultType<Self> {
        Ok(Self::Object {
            signature: value.get_signature()?,
            object: value.into_global()?,
        })
    }
}
