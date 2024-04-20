use crate::java::java_type::JavaType;
use crate::java::objects::java_object::JavaObject;
use crate::java::objects::object::GlobalJavaObject;
use crate::java::objects::value::{
    JavaBoolean, JavaByte, JavaChar, JavaDouble, JavaFloat, JavaInt, JavaLong, JavaNull, JavaShort,
    JavaValue,
};
use crate::java::traits::{GetSignature, ToJavaValue};
use crate::java::util::helpers::ResultType;
use crate::java_type::Type;
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
            JavaCallResult::Void | JavaCallResult::Null => JavaNull.into(),
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

    fn get_type(&self) -> Type {
        match self {
            JavaCallResult::Void => Type::Void,
            JavaCallResult::Null => Type::Object,
            JavaCallResult::Boolean(_) => Type::Boolean,
            JavaCallResult::Byte(_) => Type::Byte,
            JavaCallResult::Character(_) => Type::Character,
            JavaCallResult::Short(_) => Type::Short,
            JavaCallResult::Integer(_) => Type::Integer,
            JavaCallResult::Long(_) => Type::Long,
            JavaCallResult::Float(_) => Type::Float,
            JavaCallResult::Double(_) => Type::Double,
            JavaCallResult::Object { .. } => Type::Object,
        }
    }
}

impl TryFrom<JavaObject<'_>> for JavaCallResult {
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from(value: JavaObject) -> ResultType<Self> {
        Ok(Self::Object {
            signature: value.get_signature().clone(),
            object: value.into_global()?,
        })
    }
}
