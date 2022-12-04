use crate::jni::java_env::JavaEnv;
use crate::jni::jni_error::JNIError;
use crate::jni::objects::class::JavaClass;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::object::LocalJavaObject;
use crate::jni::objects::value::JavaValue;
use crate::jni::traits::{GetRaw, ToJavaValue};
use crate::jni::util::util::ResultType;
use crate::{define_array, sys};

pub struct JavaArray<'a> {
    object: LocalJavaObject<'a>,
}

impl JavaArray<'_> {
    pub fn len(&self) -> ResultType<i32> {
        self.object.env().get_array_length(unsafe { self.object.get_raw() }, )
    }
}

impl GetRaw for JavaArray<'_> {
    unsafe fn get_raw(&self) -> sys::jobject {
        self.object.get_raw()
    }
}

impl<'a> From<LocalJavaObject<'a>> for JavaArray<'a> {
    fn from(object: LocalJavaObject<'a>) -> Self {
        Self { object }
    }
}

pub struct JavaObjectArray<'a>(JavaArray<'a>);

impl<'a> JavaObjectArray<'a> {
    pub fn new(class: &'a JavaClass<'a>, length: usize) -> ResultType<Self> {
        class.env().create_object_array(class, length as i32)
    }

    pub fn from_vec(objects: Vec<Option<JavaObject<'a>>>, class: &'a JavaClass<'a>) -> ResultType<Self> {
        let mut array = JavaObjectArray::new(class, objects.len())?;
        for (i, object) in objects.into_iter().enumerate() {
            array.set(i as i32, object)?;
        }

        Ok(array)
    }

    pub unsafe fn from_raw(object: sys::jobject, env: &'a JavaEnv<'a>) -> Self {
        Self(JavaArray {
            object: LocalJavaObject::from_raw(object, env),
        })
    }

    pub fn len(&self) -> ResultType<i32> {
        self.0.len()
    }

    pub fn get(&'a self, i: i32) -> ResultType<Option<LocalJavaObject<'a>>> {
        self.get_with_errors(i, true)
    }

    pub fn set(&mut self, i: i32, value: Option<JavaObject<'a>>) -> ResultType<()> {
        self.0
            .object
            .env()
            .set_object_array_element(&self.0, i, value)
    }

    pub fn get_with_errors(
        &'a self,
        i: i32,
        resolve_errors: bool,
    ) -> ResultType<Option<LocalJavaObject<'a>>> {
        if i >= self.len()? {
            return Err(JNIError::from("Index out of bounds").into());
        }

        self.0
            .object
            .env()
            .get_object_array_element(&self.0, i, resolve_errors)
    }

    pub fn into_object(self) -> LocalJavaObject<'a> {
        self.0.object
    }
}

impl<'a> ToJavaValue<'a> for JavaObjectArray<'a> {
    fn to_java_value(&'a self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue {
            l: unsafe { self.0.object.get_raw() },
        })
    }
}

impl<'a> From<LocalJavaObject<'a>> for JavaObjectArray<'a> {
    fn from(object: LocalJavaObject<'a>) -> Self {
        Self(JavaArray::from(object))
    }
}

impl<'a> From<JavaArray<'a>> for JavaObjectArray<'a> {
    fn from(array: JavaArray<'a>) -> Self {
        Self(array)
    }
}

impl<'a> Into<LocalJavaObject<'a>> for JavaObjectArray<'a> {
    fn into(self) -> LocalJavaObject<'a> {
        self.0.object
    }
}

impl<'a> Into<JavaObject<'a>> for JavaObjectArray<'a> {
    fn into(self) -> JavaObject<'a> {
        JavaObject::from(self.0.object)
    }
}

define_array!(
    JavaShortArray,
    create_short_array,
    get_short_array_elements,
    i16
);
define_array!(JavaIntArray, create_int_array, get_int_array_elements, i32);
define_array!(
    JavaLongArray,
    create_long_array,
    get_long_array_elements,
    i64
);
define_array!(
    JavaFloatArray,
    create_float_array,
    get_float_array_elements,
    f32
);
define_array!(
    JavaDoubleArray,
    create_double_array,
    get_double_array_elements,
    f64
);
define_array!(
    JavaBooleanArray,
    create_boolean_array,
    get_boolean_array_elements,
    u8
);
define_array!(
    JavaByteArray,
    create_byte_array,
    get_byte_array_elements,
    i8
);
define_array!(
    JavaCharArray,
    create_char_array,
    get_char_array_elements,
    u16
);
