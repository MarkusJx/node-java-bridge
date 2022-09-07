use crate::jni::java_env_wrapper::JavaEnvWrapper;
use crate::jni::java_type::JavaType;
use crate::jni::java_vm::{InternalJavaOptions, JavaVM};
use crate::jni::objects::class::{GlobalJavaClass, JavaClass};
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::object::{GlobalJavaObject, LocalJavaObject};
use crate::jni::traits::{GetRaw, IsInstanceOf};
use crate::jni::util::util::{jni_version_to_string, ResultType};
use crate::jni::vm_ptr::JavaVMPtr;
use crate::{define_object_to_val_method, sys};
use std::sync::{Arc, Mutex};

/// The pointer to a java environment.
/// This should not be copied or created manually.
/// It is created by the [`JavaVM`](crate::jni::java_vm::JavaVM) struct.
/// You should also not move this between threads as it will likely
/// cause the program to segfault, or at least to panic once this is dropped,
/// since an environment is tied to a thread.
/// If you need a java environment inside a new thread, create a new on using
/// [`JavaVM::attach_thread`](crate::jni::java_vm::JavaVM::attach_thread).
pub struct JavaEnv<'a>(JavaEnvWrapper<'a>);

impl<'a> JavaEnv<'a> {
    /// You should probably not use this.
    pub(in crate::jni) fn new(
        jvm: Arc<Mutex<JavaVMPtr>>,
        options: InternalJavaOptions,
        env: *mut sys::JNIEnv,
    ) -> Self {
        Self(JavaEnvWrapper::new(jvm, options, env))
    }

    pub unsafe fn from_raw(env: *mut sys::JNIEnv, options: InternalJavaOptions) -> Self {
        Self(JavaEnvWrapper::from_raw(env, options))
    }

    pub fn get_version(&self) -> ResultType<String> {
        let version: i32 = unsafe { self.0.methods.GetVersion.unwrap()(self.0.env) };
        Ok(jni_version_to_string(version)?)
    }

    pub fn find_class(&self, class_name: &str) -> ResultType<JavaClass> {
        self.0.find_class(class_name, true)
    }

    pub fn find_class_by_java_name(&'a self, class_name: String) -> ResultType<JavaClass<'a>> {
        self.0.find_class_by_java_name(class_name)
    }

    pub fn find_global_class_by_java_name(
        &'a self,
        class_name: String,
    ) -> ResultType<GlobalJavaClass> {
        self.0.find_global_class_by_java_name(class_name)
    }

    pub fn get_java_lang_class(&'a self) -> ResultType<JavaClass<'a>> {
        self.0.get_java_lang_class()
    }

    pub fn get_system_class_loader(&self) -> ResultType<GlobalJavaObject> {
        self.0.get_system_class_loader()
    }

    pub fn get_class_loader(&self) -> ResultType<GlobalJavaObject> {
        Ok(self
            .0
            .jvm
            .as_ref()
            .ok_or("The jvm was unset".to_string())?
            .lock()
            .unwrap()
            .class_loader()
            .as_ref()
            .ok_or("The class loader was unset".to_string())?
            .clone())
    }

    pub fn get_object_class(&'a self, object: JavaObject) -> ResultType<JavaClass<'a>> {
        self.0.get_object_class(object)
    }

    pub fn get_object_signature(&self, object: JavaObject) -> ResultType<JavaType> {
        self.0.get_object_signature(object)
    }

    pub fn is_instance_of(&self, object: JavaObject, classname: &str) -> ResultType<bool> {
        self.0.is_instance_of(object, classname)
    }

    pub fn throw_error(&self, message: String) {
        self.0.throw_error(message)
    }

    pub fn throw(&self, object: JavaObject) {
        self.0.throw(object)
    }

    define_object_to_val_method!(
        object_to_int,
        i32,
        "java/lang/Integer",
        "intValue",
        "()I",
        get_int_method
    );
    define_object_to_val_method!(
        object_to_long,
        i64,
        "java/lang/Long",
        "longValue",
        "()J",
        get_long_method
    );
    define_object_to_val_method!(
        object_to_short,
        i16,
        "java/lang/Short",
        "shortValue",
        "()S",
        get_short_method
    );
    define_object_to_val_method!(
        object_to_byte,
        i8,
        "java/lang/Byte",
        "byteValue",
        "()B",
        get_byte_method
    );
    define_object_to_val_method!(
        object_to_float,
        f32,
        "java/lang/Float",
        "floatValue",
        "()F",
        get_float_method
    );
    define_object_to_val_method!(
        object_to_double,
        f64,
        "java/lang/Double",
        "doubleValue",
        "()D",
        get_double_method
    );
    define_object_to_val_method!(
        object_to_boolean,
        bool,
        "java/lang/Boolean",
        "booleanValue",
        "()Z",
        get_boolean_method
    );
    define_object_to_val_method!(
        object_to_char,
        u16,
        "java/lang/Character",
        "charValue",
        "()C",
        get_char_method
    );

    pub fn object_to_string(&self, object: &LocalJavaObject) -> ResultType<String> {
        if !object.is_instance_of("java/lang/String")? {
            return Err("The object is not a string".into());
        }

        unsafe {
            self.0.get_string_utf_chars(
                object
                    .get_raw()
                    .ok_or("Cannot convert null object to string".to_string())?,
            )
        }
    }

    pub fn instance_of(&self, this: JavaObject, other: GlobalJavaClass) -> ResultType<bool> {
        self.0.instance_of(this, other)
    }

    pub fn get_java_vm(&self) -> ResultType<JavaVM> {
        self.0.get_java_vm()
    }

    pub fn append_class_path(&self, paths: Vec<String>) -> ResultType<()> {
        self.0.append_class_path(paths)
    }

    pub(in crate::jni) fn thread_set_context_classloader(&self) -> ResultType<()> {
        if self.get_class_loader().is_err() {
            return Ok(());
        }

        let thread_class = self.find_class("java/lang/Thread")?;
        let get_current_thread = thread_class.get_static_object_method("currentThread", "()Ljava/lang/Thread;")?;
        let set_context_classloader = thread_class.get_void_method("setContextClassLoader", "(Ljava/lang/ClassLoader;)V")?;

        let current_thread = get_current_thread.call(vec![])?;
        set_context_classloader.call(JavaObject::from(current_thread), vec![Box::new(&self.get_class_loader()?)])
    }

    pub(in crate::jni) fn delete_global_ref(&self, object: sys::jobject) -> () {
        unsafe {
            self.0.methods.DeleteGlobalRef.unwrap()(self.0.env, object);
        }
    }

    pub(in crate::jni) fn get_env(&'a self) -> &'a JavaEnvWrapper<'a> {
        &self.0
    }
}
