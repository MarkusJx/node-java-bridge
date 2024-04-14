use crate::java::java_env::JavaEnv;
use crate::java::jni_error::JNIError;
use crate::java::objects::class::JavaClass;
use crate::java::objects::java_object::JavaObject;
use crate::java::objects::object::LocalJavaObject;
use crate::java::objects::value::JavaValue;
use crate::java::traits::{GetRaw, ToJavaValue};
use crate::java::util::helpers::ResultType;
use crate::java_type::{JavaType, Type};
use crate::traits::GetSignature;
use crate::{define_array, sys};

pub struct JavaArray<'a> {
    object: LocalJavaObject<'a>,
}

impl JavaArray<'_> {
    pub fn len(&self) -> ResultType<i32> {
        self.object
            .env()
            .get_array_length(unsafe { self.object.get_raw() })
    }

    pub fn is_empty(&self) -> ResultType<bool> {
        self.len().map(|len| len == 0)
    }
}

impl GetSignature for JavaArray<'_> {
    fn get_signature(&self) -> JavaType {
        self.object.get_signature()
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
        #[cfg(feature = "log")]
        crate::trace!(
            "Creating object array of type {} with length {}",
            class.get_signature(),
            length
        );

        class.env().create_object_array(class, length as i32)
    }

    pub fn from_vec(
        objects: Vec<Option<JavaObject<'a>>>,
        class: &'a JavaClass<'a>,
    ) -> ResultType<Self> {
        #[cfg(feature = "log")]
        crate::trace!(
            "Creating object array of type {} with length {}",
            class.get_signature(),
            objects.len()
        );

        let mut array = JavaObjectArray::new(class, objects.len())?;
        for (i, object) in objects.into_iter().enumerate() {
            array.set(i as i32, object)?;
        }

        Ok(array)
    }

    /// Create a new JavaObjectArray from a raw jobjectArray.
    ///
    /// # Safety
    /// This function is safe as long as the jobjectArray is a valid jobjectArray
    /// and not already owned by another JavaObjectArray.
    pub unsafe fn from_raw(
        object: sys::jobject,
        env: &'a JavaEnv<'a>,
        signature: Option<JavaType>,
    ) -> Self {
        Self(JavaArray {
            object: LocalJavaObject::from_raw(
                object,
                env,
                Some(signature.unwrap_or_else(|| JavaType::array(JavaType::object()))),
            ),
        })
    }

    pub fn len(&self) -> ResultType<i32> {
        self.0.len()
    }

    pub fn is_empty(&self) -> ResultType<bool> {
        self.0.is_empty()
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

impl GetSignature for JavaObjectArray<'_> {
    fn get_signature(&self) -> JavaType {
        self.0.get_signature()
    }
}

impl<'a> ToJavaValue<'a> for JavaObjectArray<'a> {
    fn to_java_value(&'a self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue {
            l: unsafe { self.0.object.get_raw() },
        })
    }

    fn get_type(&self) -> Type {
        Type::Array
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

impl<'a> From<JavaObjectArray<'a>> for LocalJavaObject<'a> {
    fn from(value: JavaObjectArray<'a>) -> LocalJavaObject<'a> {
        value.0.object
    }
}

impl<'a> From<JavaObjectArray<'a>> for JavaObject<'a> {
    fn from(value: JavaObjectArray<'a>) -> JavaObject<'a> {
        JavaObject::from(value.0.object)
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
