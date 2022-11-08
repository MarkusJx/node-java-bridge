use crate::jni::java_call_result::JavaCallResult;
use crate::jni::java_env::JavaEnv;
use crate::jni::java_type::{JavaType, Type};
use crate::jni::objects::class::{GlobalJavaClass, JavaClass};
use crate::jni::objects::java_object::JavaObject;
use crate::jni::traits::IsNull;
use crate::jni::util::util::ResultType;
use crate::{define_field, sys};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct JavaField<'a> {
    field: sys::jfieldID,
    class: &'a JavaClass<'a>,
    field_type: JavaType,
    _marker: PhantomData<&'a sys::jfieldID>,
    is_static: bool,
}

impl<'a> JavaField<'a> {
    pub(in crate::jni) unsafe fn new(
        field: sys::jfieldID,
        field_type: JavaType,
        class: &'a JavaClass<'a>,
        is_static: bool,
    ) -> Self {
        Self {
            field,
            class,
            field_type,
            _marker: PhantomData,
            is_static,
        }
    }

    pub(in crate::jni) unsafe fn id(&self) -> sys::jfieldID {
        self.field
    }
}

pub trait JavaFieldValues {
    fn set(&self, object: &JavaObject<'_>, value: JavaCallResult) -> ResultType<()>;

    fn get(&self, object: &JavaObject<'_>) -> ResultType<JavaCallResult>;
}

pub trait StaticJavaFieldValues {
    fn set(&self, value: JavaCallResult) -> ResultType<()>;

    fn get(&self) -> ResultType<JavaCallResult>;
}

define_field!(
    JavaIntField,
    StaticJavaIntField,
    i32,
    get_int_field,
    set_int_field,
    get_static_int_field,
    set_static_int_field,
    Integer,
    value,
    value,
    value,
    &[Type::Integer]
);
define_field!(
    JavaLongField,
    StaticJavaLongField,
    i64,
    get_long_field,
    set_long_field,
    get_static_long_field,
    set_static_long_field,
    Long,
    value,
    value,
    value,
    &[Type::Long]
);
define_field!(
    JavaFloatField,
    StaticJavaFloatField,
    f32,
    get_float_field,
    set_float_field,
    get_static_float_field,
    set_static_float_field,
    Float,
    value,
    value,
    value,
    &[Type::Float]
);
define_field!(
    JavaDoubleField,
    StaticJavaDoubleField,
    f64,
    get_double_field,
    set_double_field,
    get_static_double_field,
    set_static_double_field,
    Double,
    value,
    value,
    value,
    &[Type::Double]
);
define_field!(
    JavaShortField,
    StaticJavaShortField,
    i16,
    get_short_field,
    set_short_field,
    get_static_short_field,
    set_static_short_field,
    Short,
    value,
    value,
    value,
    &[Type::Short]
);
define_field!(
    JavaByteField,
    StaticJavaByteField,
    i8,
    get_byte_field,
    set_byte_field,
    get_static_byte_field,
    set_static_byte_field,
    Byte,
    value,
    value,
    value,
    &[Type::Byte]
);
define_field!(
    JavaBooleanField,
    StaticJavaBooleanField,
    u8,
    get_boolean_field,
    set_boolean_field,
    get_static_boolean_field,
    set_static_boolean_field,
    Boolean,
    value,
    value as u8,
    value != 0,
    &[Type::Boolean]
);
define_field!(
    JavaCharField,
    StaticJavaCharField,
    u16,
    get_char_field,
    set_char_field,
    get_static_char_field,
    set_static_char_field,
    Character,
    value,
    value,
    value,
    &[Type::Character]
);

const OBJECT_FIELD_ALLOWED_TYPES: &[Type] = &[
    Type::Object,
    Type::LangObject,
    Type::String,
    Type::Array,
    Type::LangInteger,
    Type::LangLong,
    Type::LangFloat,
    Type::LangDouble,
    Type::LangShort,
    Type::LangByte,
    Type::LangBoolean,
    Type::LangCharacter,
];

pub struct JavaObjectField<'a>(JavaField<'a>);

impl<'a> JavaObjectField<'a> {
    pub fn new(field: JavaField<'a>) -> Self {
        Self(field)
    }

    pub fn get(&self, object: &JavaObject<'_>) -> ResultType<JavaObject> {
        self.0.class.env().get_object_field(&self, object)
    }

    pub fn set(&self, object: &JavaObject<'_>, value: JavaObject) -> ResultType<()> {
        self.0.class.env().set_object_field(self, object, value)
    }

    pub fn from_global(field: GlobalJavaField, class: &'a JavaClass<'a>) -> ResultType<Self> {
        if field.is_static {
            return Err("Tried creating a non-static field from a static field".into());
        }

        let t = field.field_type.type_enum();
        if !OBJECT_FIELD_ALLOWED_TYPES.contains(&t) {
            return Err(format!("{} is not a valid type for this field", t).into());
        }

        unsafe {
            Ok(Self(JavaField::new(
                field.field.load(Ordering::Relaxed),
                field.field_type,
                &class,
                field.is_static,
            )))
        }
    }

    pub(in crate::jni) unsafe fn id(&self) -> sys::jfieldID {
        self.0.id()
    }
}

fn get_field_value(object: JavaObject, inner: &JavaField) -> ResultType<JavaCallResult> {
    Ok(if object.is_null() {
        JavaCallResult::Null
    } else {
        JavaCallResult::Object {
            object: object.into_global()?,
            signature: inner.field_type.clone(),
        }
    })
}

impl<'a> JavaFieldValues for JavaObjectField<'a> {
    fn set(&self, obj: &JavaObject<'_>, value: JavaCallResult) -> ResultType<()> {
        match value {
            JavaCallResult::Object { object, .. } => self.set(obj, JavaObject::from(object)),
            _ => Err("Invalid value type supplied for field StaticJavaObjectField".into()),
        }
    }

    fn get(&self, obj: &JavaObject<'_>) -> ResultType<JavaCallResult> {
        let object = self.get(obj)?;
        get_field_value(object, &self.0)
    }
}

pub struct StaticJavaObjectField<'a>(JavaField<'a>);

impl<'a> StaticJavaObjectField<'a> {
    pub fn new(field: JavaField<'a>) -> Self {
        Self(field)
    }

    pub fn get(&self) -> ResultType<JavaObject> {
        self.0
            .class
            .env()
            .get_static_object_field(self, self.0.class)
    }

    pub fn set(&self, value: JavaObject) -> ResultType<()> {
        self.0
            .class
            .env()
            .set_static_object_field(self, self.0.class, value)
    }

    pub fn from_global(field: GlobalJavaField, class: &'a JavaClass<'a>) -> ResultType<Self> {
        if !field.is_static {
            return Err("Tried creating a static field from a non-static field".into());
        }

        let t = field.field_type.type_enum();
        if !OBJECT_FIELD_ALLOWED_TYPES.contains(&t) {
            return Err(format!("{} is not a valid type for this field", t).into());
        }

        unsafe {
            Ok(Self(JavaField::new(
                field.field.load(Ordering::Relaxed),
                field.field_type,
                &class,
                field.is_static,
            )))
        }
    }

    pub(in crate::jni) unsafe fn id(&self) -> sys::jfieldID {
        self.0.id()
    }
}

impl<'a> StaticJavaFieldValues for StaticJavaObjectField<'a> {
    fn set(&self, value: JavaCallResult) -> ResultType<()> {
        match value {
            JavaCallResult::Object { object, .. } => self.set(JavaObject::from(object)),
            _ => Err("Invalid value type supplied for field StaticJavaObjectField".into()),
        }
    }

    fn get(&self) -> ResultType<JavaCallResult> {
        let object = self.get()?;
        get_field_value(object, &self.0)
    }
}

pub struct GlobalJavaField {
    field: AtomicPtr<sys::_jfieldID>,
    field_type: JavaType,
    class: GlobalJavaClass,
    is_static: bool,
}

impl GlobalJavaField {
    pub fn from(field: JavaField, class: GlobalJavaClass) -> Self {
        Self {
            field: AtomicPtr::new(field.field),
            field_type: field.field_type,
            class,
            is_static: field.is_static,
        }
    }

    pub fn get_class<'a>(&'a self, env: &'a JavaEnv<'a>) -> JavaClass<'a> {
        JavaClass::from_global(&self.class, env)
    }

    pub fn to_java_field<'a>(
        &'a self,
        class: &'a JavaClass<'a>,
    ) -> ResultType<Box<dyn JavaFieldValues + 'a>> {
        Ok(match self.field_type.type_enum() {
            Type::Integer => Box::new(JavaIntField::from_global(self.clone(), class)?),
            Type::Long => Box::new(JavaLongField::from_global(self.clone(), class)?),
            Type::Float => Box::new(JavaFloatField::from_global(self.clone(), class)?),
            Type::Double => Box::new(JavaDoubleField::from_global(self.clone(), class)?),
            Type::Short => Box::new(JavaShortField::from_global(self.clone(), class)?),
            Type::Byte => Box::new(JavaByteField::from_global(self.clone(), class)?),
            Type::Boolean => Box::new(JavaBooleanField::from_global(self.clone(), class)?),
            Type::Character => Box::new(JavaCharField::from_global(self.clone(), class)?),
            Type::Void => return Err("Void is not a valid type for a field".into()),
            _ => Box::new(JavaObjectField::from_global(self.clone(), class)?),
        })
    }

    pub fn to_static_java_field<'a>(
        &'a self,
        class: &'a JavaClass<'a>,
    ) -> ResultType<Box<dyn StaticJavaFieldValues + 'a>> {
        Ok(match self.field_type.type_enum() {
            Type::Integer => Box::new(StaticJavaIntField::from_global(self.clone(), class)?),
            Type::Long => Box::new(StaticJavaLongField::from_global(self.clone(), class)?),
            Type::Float => Box::new(StaticJavaFloatField::from_global(self.clone(), class)?),
            Type::Double => Box::new(StaticJavaDoubleField::from_global(self.clone(), class)?),
            Type::Short => Box::new(StaticJavaShortField::from_global(self.clone(), class)?),
            Type::Byte => Box::new(StaticJavaByteField::from_global(self.clone(), class)?),
            Type::Boolean => Box::new(StaticJavaBooleanField::from_global(self.clone(), class)?),
            Type::Character => Box::new(StaticJavaCharField::from_global(self.clone(), class)?),
            Type::Void => return Err("Void is not a valid type for a field".into()),
            _ => Box::new(StaticJavaObjectField::from_global(self.clone(), class)?),
        })
    }

    pub fn get_type(&self) -> &JavaType {
        &self.field_type
    }
}

impl Clone for GlobalJavaField {
    fn clone(&self) -> Self {
        Self {
            field: AtomicPtr::new(self.field.load(Ordering::Relaxed)),
            field_type: self.field_type.clone(),
            class: self.class.clone(),
            is_static: self.is_static,
        }
    }
}
