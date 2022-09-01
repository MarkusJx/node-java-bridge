use crate::jni::java_env::JavaEnv;
use crate::jni::java_env_wrapper::JavaEnvWrapper;
use crate::jni::java_type::JavaType;
use crate::jni::java_vm::{InternalJavaOptions, JavaVM};
use crate::jni::jni_error::JNIError;
use crate::jni::objects::args::JavaArg;
use crate::jni::objects::class::JavaClass;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::string::JavaString;
use crate::jni::objects::value::{
    JavaBoolean, JavaByte, JavaChar, JavaDouble, JavaFloat, JavaInt, JavaLong, JavaShort, JavaValue,
};
use crate::jni::traits::{GetRaw, GetSignature, IsInstanceOf, IsNull, ToJavaValue};
use crate::jni::util::util::ResultType;
use crate::jni::vm_ptr::JavaVMPtr;
use crate::{define_object_value_of_method, sys};
use std::error::Error;
use std::marker::PhantomData;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

pub struct LocalJavaObject<'a> {
    object: sys::jobject,
    free: bool,
    env: &'a JavaEnvWrapper<'a>,
    _marker: PhantomData<&'a sys::jobject>,
}

impl<'a> LocalJavaObject<'a> {
    pub(in crate::jni) fn new(object: sys::jobject, env: &'a JavaEnvWrapper<'a>) -> Self {
        Self {
            object,
            free: true,
            env,
            _marker: PhantomData,
        }
    }

    pub unsafe fn from_raw(object: sys::jobject, env: &'a JavaEnv<'a>) -> Self {
        Self {
            object,
            free: true,
            env: env.get_env(),
            _marker: PhantomData,
        }
    }

    pub fn to_java_string(self) -> JavaString<'a> {
        JavaString::from(self)
    }

    pub fn from(object: &'a GlobalJavaObject, env: &'a JavaEnv<'a>) -> Self {
        Self {
            object: object.0.lock().unwrap().object.load(Ordering::Relaxed),
            free: false,
            env: env.get_env(),
            _marker: PhantomData,
        }
    }

    pub(in crate::jni) fn from_internal(
        object: &'a GlobalJavaObject,
        env: &'a JavaEnvWrapper<'a>,
    ) -> Self {
        Self {
            object: object.0.lock().unwrap().object.load(Ordering::Relaxed),
            free: false,
            env,
            _marker: PhantomData,
        }
    }

    pub(in crate::jni) fn assign_env<'b>(
        mut self,
        env: &'b JavaEnvWrapper<'b>,
    ) -> LocalJavaObject<'b> {
        let free = self.free;
        self.free = false;

        LocalJavaObject {
            object: self.object,
            free,
            env,
            _marker: PhantomData,
        }
    }

    pub(in crate::jni) fn env(&'a self) -> &'a JavaEnvWrapper<'a> {
        &self.env
    }

    define_object_value_of_method!(
        /// This specific method creates a new `java.lang.Integer` from ``i32``.
        /// # Example
        /// ```rust
        /// let lang_int = LocalJavaObject::from_i32(&env, 42)?;
        ///
        /// // Convert the value back to i32
        /// let original_value = env.object_to_int(&lang_int)?;
        /// assert_eq!(original_value, 42);
        /// ```
        => from_i32, "java/lang/Integer", "I", i32, JavaInt
    );

    define_object_value_of_method!(
        => from_i64, "java/lang/Long", "J", i64, JavaLong
    );

    define_object_value_of_method!(
        => from_f32, "java/lang/Float", "F", f32, JavaFloat
    );

    define_object_value_of_method!(
        => from_f64, "java/lang/Double", "D", f64, JavaDouble
    );

    define_object_value_of_method!(
        => from_bool, "java/lang/Boolean", "Z", bool, JavaBoolean
    );

    define_object_value_of_method!(
        => from_char, "java/lang/Character", "C", u16, JavaChar
    );

    define_object_value_of_method!(
        => from_i16, "java/lang/Short", "S", i16, JavaShort
    );

    define_object_value_of_method!(
        => from_byte, "java/lang/Byte", "B", i8, JavaByte
    );
}

impl GetRaw for LocalJavaObject<'_> {
    unsafe fn get_raw_nullable(&self) -> sys::jobject {
        self.object
    }
}

impl IsNull for LocalJavaObject<'_> {
    fn is_null(&self) -> bool {
        self.object == ptr::null_mut()
    }
}

impl<'a> IsInstanceOf for LocalJavaObject<'a> {
    fn is_instance_of(&self, classname: &str) -> ResultType<bool> {
        self.env.is_instance_of(JavaObject::from(self), classname)
    }
}

impl<'a> GetSignature for LocalJavaObject<'a> {
    fn get_signature(&self) -> ResultType<JavaType> {
        self.env.get_object_signature(JavaObject::from(self))
    }
}

impl<'a> ToJavaValue<'a> for LocalJavaObject<'a> {
    fn to_java_value(&'a self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue { l: self.object })
    }
}

impl<'a> From<&'a LocalJavaObject<'a>> for JavaArg<'a> {
    fn from(object: &'a LocalJavaObject<'a>) -> Self {
        Box::new(object)
    }
}

impl<'a> Drop for LocalJavaObject<'a> {
    fn drop(&mut self) {
        if self.free && !self.is_null() {
            self.env.delete_local_ref(self.object);
        }
    }
}

struct GlobalJavaObjectInternal {
    object: AtomicPtr<sys::_jobject>,
    jvm: Arc<Mutex<JavaVMPtr>>,
    options: InternalJavaOptions,
    free: bool,
}

impl GlobalJavaObjectInternal {
    pub fn new(
        object: sys::jobject,
        jvm: Arc<Mutex<JavaVMPtr>>,
        options: InternalJavaOptions,
    ) -> Self {
        Self {
            object: AtomicPtr::new(object),
            jvm,
            options,
            free: true,
        }
    }

    fn get_vm(&self) -> JavaVM {
        JavaVM::from_existing(self.jvm.clone(), self.options)
    }

    fn disable_free(&mut self) {
        self.free = false
    }
}

impl Drop for GlobalJavaObjectInternal {
    fn drop(&mut self) {
        if self.free {
            let vm = JavaVM::from_existing(self.jvm.clone(), self.options);
            let env = vm.attach_thread();

            if env.is_ok() {
                env.unwrap()
                    .delete_global_ref(self.object.load(Ordering::Relaxed));
            }
        }
    }
}

#[derive(Clone)]
pub struct GlobalJavaObject(Arc<Mutex<GlobalJavaObjectInternal>>);

impl GlobalJavaObject {
    pub fn null(env: &JavaEnv) -> ResultType<Self> {
        Ok(Self(Arc::new(Mutex::new(GlobalJavaObjectInternal {
            object: AtomicPtr::new(ptr::null_mut()),
            jvm: env.get_env().get_vm_ptr()?,
            options: env.get_env().get_options()?,
            free: true,
        }))))
    }

    pub fn new(
        object: sys::jobject,
        jvm: Arc<Mutex<JavaVMPtr>>,
        options: InternalJavaOptions,
    ) -> Self {
        Self(Arc::new(Mutex::new(GlobalJavaObjectInternal::new(
            object, jvm, options,
        ))))
    }

    pub fn get_class<'a>(&self, env: &'a JavaEnv<'a>) -> ResultType<JavaClass<'a>> {
        if self.is_null() {
            return Err("Cannot get class for null pointer".into());
        }

        env.get_object_class(JavaObject::from(self))
    }

    /// Get this object's raw value in order to pass it
    /// to the JVM as a method return value.
    /// Disables automatic freeing of the object
    /// and allows the returned value to be `null`.
    pub unsafe fn into_return_value(self) -> sys::jobject {
        self.0.lock().unwrap().disable_free();
        self.get_raw_nullable()
    }
}

impl IsNull for GlobalJavaObject {
    fn is_null(&self) -> bool {
        self.0
            .lock()
            .unwrap()
            .object
            .load(Ordering::Relaxed)
            .is_null()
    }
}

impl IsInstanceOf for GlobalJavaObject {
    fn is_instance_of(&self, classname: &str) -> ResultType<bool> {
        if self.is_null() {
            return Err("Cannot check instance of null pointer".into());
        }

        let vm = self.0.lock().unwrap().get_vm();
        let env = vm.attach_thread()?;

        env.is_instance_of(JavaObject::from(self), classname)
    }
}

impl GetRaw for GlobalJavaObject {
    unsafe fn get_raw_nullable(&self) -> sys::jobject {
        self.0.lock().unwrap().object.load(Ordering::Relaxed)
    }
}

impl GetSignature for GlobalJavaObject {
    fn get_signature(&self) -> ResultType<JavaType> {
        if self.is_null() {
            return Err("Cannot get signature for null pointer".into());
        }

        let vm = self.0.lock().unwrap().get_vm();
        let env = vm.attach_thread()?;

        env.get_env().get_object_signature(JavaObject::from(self))
    }
}

impl<'a> ToJavaValue<'a> for GlobalJavaObject {
    fn to_java_value(&'a self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue {
            l: self.0.lock().unwrap().object.load(Ordering::Relaxed),
        })
    }
}

impl<'a> TryFrom<LocalJavaObject<'a>> for GlobalJavaObject {
    type Error = Box<dyn Error>;

    fn try_from(mut local: LocalJavaObject<'a>) -> Result<GlobalJavaObject, Self::Error> {
        if local.is_null() {
            return Err(JNIError::from("Cannot convert null local object to global object").into());
        }

        local.free = false;
        local.env.new_global_object(local.object)
    }
}

impl<'a> TryFrom<JavaString<'a>> for GlobalJavaObject {
    type Error = Box<dyn Error>;

    fn try_from(mut string: JavaString<'a>) -> Result<GlobalJavaObject, Self::Error> {
        if string.0.is_null() {
            return Err(JNIError::from("Cannot convert null local object to global object").into());
        }

        string.0.free = false;
        string.0.env.new_global_object(string.0.object)
    }
}

unsafe impl Send for GlobalJavaObject {}
