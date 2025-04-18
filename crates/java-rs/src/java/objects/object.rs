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
use crate::java::util::helpers::ResultType;
use crate::java::vm_ptr::JavaVMPtr;
use crate::java_type::Type;
use crate::objects::java_object::AsJavaObject;
use crate::{assert_non_null, define_object_value_of_method, sys};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

pub struct LocalJavaObject<'a> {
    object: sys::jobject,
    free: bool,
    env: &'a JavaEnvWrapper<'a>,
    #[cfg(feature = "type_check")]
    signature: JavaType,
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
        }
    }

    /// Create a new local java object from a raw pointer.
    ///
    /// # Safety
    /// This function is unsafe as it creates a new local reference from a raw pointer.
    /// The object will be deleted when the local reference is deleted.
    /// This assumes the pointer is actually a valid object pointer and not already
    /// owned by another local reference.
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
        }
    }

    pub(in crate::java) fn env(&'a self) -> &'a JavaEnvWrapper<'a> {
        self.env
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
            self.get_signature()
        )
    }
}

impl GetRaw for LocalJavaObject<'_> {
    unsafe fn get_raw(&self) -> sys::jobject {
        self.object
    }
}

impl IsInstanceOf for LocalJavaObject<'_> {
    fn is_instance_of(&self, classname: &str) -> ResultType<bool> {
        self.env.is_instance_of(JavaObject::from(self), classname)
    }
}

impl GetSignature for LocalJavaObject<'_> {
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

impl Drop for LocalJavaObject<'_> {
    fn drop(&mut self) {
        if self.free {
            self.env.delete_local_ref(self.object);
        }
    }
}

struct GlobalJavaObjectInternal {
    object: AtomicPtr<sys::_jobject>,
    jvm: Arc<Mutex<JavaVMPtr>>,
}

impl GlobalJavaObjectInternal {
    pub fn new(object: sys::jobject, jvm: Arc<Mutex<JavaVMPtr>>) -> Self {
        assert_non_null!(object, "GlobalJavaObject::new: object is null");

        Self {
            object: AtomicPtr::new(object),
            jvm,
        }
    }

    fn get_vm(&self) -> JavaVM {
        JavaVM::from_existing(self.jvm.clone())
    }
}

impl Drop for GlobalJavaObjectInternal {
    fn drop(&mut self) {
        let vm = JavaVM::from_existing(self.jvm.clone());
        let env = vm.attach_thread();

        if let Ok(env) = env {
            env.delete_global_ref(self.object.load(Ordering::Relaxed));
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

    /// Get this objects raw value in order to pass it
    /// to the JVM as a method return value.
    /// Creates a new local reference to the object
    /// and allows the returned value to be `null`.
    ///
    /// # Safety
    /// This method converts the object to a local reference
    /// and returns the raw pointer to the local reference.
    /// The local reference either has to be destroyed manually
    /// or will be destroyed when the local reference is returned
    /// to the JVM.
    pub unsafe fn into_return_value(self, env: &JavaEnv) -> sys::jobject {
        env.create_local_ref(self.get_raw())
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
            self.get_signature()
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
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from(local: LocalJavaObject<'a>) -> Result<GlobalJavaObject, Self::Error> {
        local.env.new_global_object(
            local.object,
            #[cfg(feature = "type_check")]
            local.signature.clone(),
        )
    }
}

impl<'a> TryFrom<JavaString<'a>> for GlobalJavaObject {
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from(string: JavaString<'a>) -> Result<GlobalJavaObject, Self::Error> {
        string.0.env.new_global_object(
            string.0.object,
            #[cfg(feature = "type_check")]
            JavaType::string(),
        )
    }
}

unsafe impl Send for GlobalJavaObject {}
