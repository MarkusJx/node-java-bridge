use crate::java::java_env::JavaEnv;
use crate::java::java_env_wrapper::JavaEnvWrapper;
use crate::java::java_type::JavaType;
use crate::java::java_vm::JavaVM;
use crate::java::objects::args::JavaArg;
use crate::java::objects::class::JavaClass;
use crate::java::objects::java_object::JavaObject;
use crate::java::objects::string::JavaString;
use crate::java::objects::value::{
    JavaBoolean, JavaByte, JavaChar, JavaDouble, JavaFloat, JavaInt, JavaLong, JavaShort, JavaValue,
};
use crate::java::traits::{GetRaw, GetSignature, IsInstanceOf, ToJavaValue};
use crate::java::util::util::ResultType;
use crate::java::vm_ptr::JavaVMPtr;
use crate::java_type::Type;
use crate::objects::java_object::AsJavaObject;
use crate::{assert_non_null, define_object_value_of_method, sys};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

pub struct LocalJavaObject<'a> {
    object: sys::jobject,
    free: bool,
    env: &'a JavaEnvWrapper<'a>,
    #[cfg(feature = "type_check")]
    signature: JavaType,
    _marker: PhantomData<&'a sys::jobject>,
}

impl<'a> LocalJavaObject<'a> {
    pub(in crate::java) fn new(
        object: sys::jobject,
        env: &'a JavaEnvWrapper<'a>,
        #[cfg(feature = "type_check")] signature: JavaType,
    ) -> Self {
        assert_non_null!(object, "LocalJavaObject::new: object is null");

        Self {
            object,
            free: true,
            env,
            #[cfg(feature = "type_check")]
            signature,
            _marker: PhantomData,
        }
    }

    pub unsafe fn from_raw(
        object: sys::jobject,
        env: &'a JavaEnv<'a>,
        #[cfg(feature = "type_check")] signature: Option<JavaType>,
        #[cfg(not(feature = "type_check"))] _signature: Option<JavaType>,
    ) -> Self {
        assert_non_null!(object, "LocalJavaObject::from_raw: object is null");

        Self {
            object,
            #[cfg(feature = "type_check")]
            signature: signature.unwrap_or(JavaType::object()),
            free: true,
            env: env.get_env(),
            _marker: PhantomData,
        }
    }

    pub fn to_java_string(self) -> ResultType<JavaString<'a>> {
        JavaString::try_from(self)
    }

    pub fn from(object: &'a GlobalJavaObject, env: &'a JavaEnv<'a>) -> Self {
        #[cfg(feature = "type_check")]
        crate::trace!(
            "Creating local java object from global java object with signature: {}",
            object.signature
        );

        let inner = object.object.lock().unwrap();
        Self {
            object: inner.object.load(Ordering::Relaxed),
            free: false,
            env: env.get_env(),
            #[cfg(feature = "type_check")]
            signature: object.signature.clone(),
            _marker: PhantomData,
        }
    }

    pub(in crate::java) fn from_internal(
        object: &'a GlobalJavaObject,
        env: &'a JavaEnvWrapper<'a>,
    ) -> Self {
        let inner = object.object.lock().unwrap();
        Self {
            object: inner.object.load(Ordering::Relaxed),
            free: false,
            env,
            #[cfg(feature = "type_check")]
            signature: object.signature.clone(),
            _marker: PhantomData,
        }
    }

    pub(in crate::java) fn assign_env<'b>(
        mut self,
        env: &'b JavaEnvWrapper<'b>,
    ) -> LocalJavaObject<'b> {
        assert_non_null!(self.object);
        let free = self.free;
        self.free = false;

        LocalJavaObject {
            object: self.object,
            free,
            env,
            #[cfg(feature = "type_check")]
            signature: self.signature.clone(),
            _marker: PhantomData,
        }
    }

    pub(in crate::java) fn env(&'a self) -> &'a JavaEnvWrapper<'a> {
        &self.env
    }

    define_object_value_of_method!(
        /// This specific method creates a new `java.lang.Integer` from ``i32``.
        /// # Example
        /// ```rust
        /// use java_rs::java_vm::{JavaVM};
        /// use java_rs::objects::object::LocalJavaObject;
        ///
        /// let env = JavaVM::new(
        ///     &"1.8".to_string(),
        ///     None,
        ///     &vec![],
        /// )
        /// .unwrap()
        /// .attach_thread()
        /// .unwrap();
        /// let lang_int = LocalJavaObject::from_i32(&env, 42).unwrap();
        ///
        /// // Convert the value back to i32
        /// let original_value = env.object_to_int(&lang_int).unwrap();
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

impl<'a> AsJavaObject<'a> for LocalJavaObject<'a> {
    fn as_java_object(&'a self) -> JavaObject<'a> {
        JavaObject::LocalRef(self)
    }
}

impl Debug for LocalJavaObject<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LocalJavaObject(object: {}, signature: {})",
            unsafe { self.get_raw() } as usize,
            self.get_signature().to_string()
        )
    }
}

impl GetRaw for LocalJavaObject<'_> {
    unsafe fn get_raw(&self) -> sys::jobject {
        self.object
    }
}

impl<'a> IsInstanceOf for LocalJavaObject<'a> {
    fn is_instance_of(&self, classname: &str) -> ResultType<bool> {
        self.env.is_instance_of(JavaObject::from(self), classname)
    }
}

impl<'a> GetSignature for LocalJavaObject<'a> {
    #[cfg(feature = "type_check")]
    fn get_signature(&self) -> JavaType {
        self.signature.clone()
    }

    #[cfg(not(feature = "type_check"))]
    fn get_signature(&self) -> JavaType {
        self.env
            .get_object_signature(JavaObject::from(self))
            .unwrap()
    }
}

impl<'a> ToJavaValue<'a> for LocalJavaObject<'a> {
    fn to_java_value(&'a self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue { l: self.object })
    }

    fn get_type(&self) -> Type {
        Type::Object
    }
}

impl<'a> From<&'a LocalJavaObject<'a>> for JavaArg<'a> {
    fn from(object: &'a LocalJavaObject<'a>) -> Self {
        Box::new(object)
    }
}

impl<'a> Drop for LocalJavaObject<'a> {
    fn drop(&mut self) {
        if self.free {
            self.env.delete_local_ref(self.object);
        }
    }
}

struct GlobalJavaObjectInternal {
    object: AtomicPtr<sys::_jobject>,
    jvm: Arc<Mutex<JavaVMPtr>>,
    free: bool,
}

impl GlobalJavaObjectInternal {
    pub fn new(object: sys::jobject, jvm: Arc<Mutex<JavaVMPtr>>) -> Self {
        assert_non_null!(object, "GlobalJavaObject::new: object is null");

        Self {
            object: AtomicPtr::new(object),
            jvm,
            free: true,
        }
    }

    fn get_vm(&self) -> JavaVM {
        JavaVM::from_existing(self.jvm.clone())
    }

    fn disable_free(&mut self) {
        self.free = false
    }
}

impl Drop for GlobalJavaObjectInternal {
    fn drop(&mut self) {
        if self.free {
            let vm = JavaVM::from_existing(self.jvm.clone());
            let env = vm.attach_thread();

            if let Ok(env) = env {
                env.delete_global_ref(self.object.load(Ordering::Relaxed));
            }
        }
    }
}

#[derive(Clone)]
pub struct GlobalJavaObject {
    object: Arc<Mutex<GlobalJavaObjectInternal>>,
    #[cfg(feature = "type_check")]
    signature: JavaType,
}

impl GlobalJavaObject {
    pub fn new(
        object: sys::jobject,
        jvm: Arc<Mutex<JavaVMPtr>>,
        #[cfg(feature = "type_check")] signature: JavaType,
    ) -> Self {
        assert_non_null!(object, "GlobalJavaObject::new: object is null");

        Self {
            object: Arc::new(Mutex::new(GlobalJavaObjectInternal::new(object, jvm))),
            #[cfg(feature = "type_check")]
            signature,
        }
    }

    pub fn get_class<'a>(&self, env: &'a JavaEnv<'a>) -> ResultType<JavaClass<'a>> {
        env.get_object_class(self.into())
    }

    pub fn get_vm(&self) -> JavaVM {
        self.object.lock().unwrap().get_vm()
    }

    /// Get this object's raw value in order to pass it
    /// to the JVM as a method return value.
    /// Disables automatic freeing of the object
    /// and allows the returned value to be `null`.
    pub unsafe fn into_return_value(self) -> sys::jobject {
        self.object.lock().unwrap().disable_free();
        self.get_raw()
    }
}

impl<'a> AsJavaObject<'a> for GlobalJavaObject {
    fn as_java_object(&self) -> JavaObject<'a> {
        JavaObject::Global(self.clone())
    }
}

impl Debug for GlobalJavaObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GlobalJavaObject(object: {}, signature: {})",
            unsafe { self.get_raw() } as usize,
            self.get_signature().to_string()
        )
    }
}

impl GetSignature for GlobalJavaObject {
    #[cfg(feature = "type_check")]
    fn get_signature(&self) -> JavaType {
        self.signature.clone()
    }

    #[cfg(not(feature = "type_check"))]
    fn get_signature(&self) -> JavaType {
        self.get_vm()
            .attach_thread()
            .unwrap()
            .get_object_signature(JavaObject::from(self))
            .unwrap()
    }
}

impl IsInstanceOf for GlobalJavaObject {
    fn is_instance_of(&self, classname: &str) -> ResultType<bool> {
        let vm = self.object.lock().unwrap().get_vm();
        let env = vm.attach_thread()?;

        env.is_instance_of(self.into(), classname)
    }
}

impl GetRaw for GlobalJavaObject {
    unsafe fn get_raw(&self) -> sys::jobject {
        self.object.lock().unwrap().object.load(Ordering::Relaxed)
    }
}

impl<'a> ToJavaValue<'a> for GlobalJavaObject {
    fn to_java_value(&'a self) -> JavaValue<'a> {
        JavaValue::new(sys::jvalue {
            l: self.object.lock().unwrap().object.load(Ordering::Relaxed),
        })
    }

    fn get_type(&self) -> Type {
        Type::Object
    }
}

impl<'a> TryFrom<LocalJavaObject<'a>> for GlobalJavaObject {
    type Error = Box<dyn Error>;

    fn try_from(mut local: LocalJavaObject<'a>) -> Result<GlobalJavaObject, Self::Error> {
        local.free = false;
        local.env.new_global_object(
            local.object,
            #[cfg(feature = "type_check")]
            local.signature.clone(),
        )
    }
}

impl<'a> TryFrom<JavaString<'a>> for GlobalJavaObject {
    type Error = Box<dyn Error>;

    fn try_from(mut string: JavaString<'a>) -> Result<GlobalJavaObject, Self::Error> {
        string.0.free = false;
        string.0.env.new_global_object(
            string.0.object,
            #[cfg(feature = "type_check")]
            JavaType::string(),
        )
    }
}

unsafe impl Send for GlobalJavaObject {}
