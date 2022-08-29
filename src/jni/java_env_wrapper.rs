use crate::jni::java_error::JavaError;
use crate::jni::java_field::{
    JavaBooleanField, JavaByteField, JavaCharField, JavaDoubleField, JavaField, JavaFloatField,
    JavaIntField, JavaLongField, JavaObjectField, JavaShortField, StaticJavaBooleanField,
    StaticJavaByteField, StaticJavaCharField, StaticJavaDoubleField, StaticJavaFloatField,
    StaticJavaIntField, StaticJavaLongField, StaticJavaObjectField, StaticJavaShortField,
};
use crate::jni::java_type::JavaType;
use crate::jni::java_vm::{InternalJavaOptions, JavaVM};
use crate::jni::jni_error::JNIError;
use crate::jni::objects::args::JavaArgs;
use crate::jni::objects::array::{
    JavaArray, JavaBooleanArray, JavaByteArray, JavaCharArray, JavaDoubleArray, JavaFloatArray,
    JavaIntArray, JavaLongArray, JavaObjectArray, JavaShortArray,
};
use crate::jni::objects::class::{GlobalJavaClass, JavaClass};
use crate::jni::objects::constructor::JavaConstructor;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::method::{JavaMethod, JavaObjectMethod};
use crate::jni::objects::object::{GlobalJavaObject, LocalJavaObject};
use crate::jni::objects::string::JavaString;
use crate::jni::objects::value::JavaBoolean;
use crate::jni::traits::{GetRaw, IsNull};
use crate::jni::util::util::{jni_error_to_string, ResultType};
use crate::jni::vm_ptr::JavaVMPtr;
use crate::{define_array_methods, define_call_methods, define_field_methods, sys};
use std::borrow::Borrow;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

/// A wrapper around a JNIEnv.
/// This manages the reference count for the current thread
/// and provides convenience methods to the jvm.
/// Rather than copying or creating this directly, attach a new
/// thread using [`JavaVM::attach_thread`](crate::jni::java_vm::JavaVM::attach_thread).
pub struct JavaEnvWrapper<'a> {
    pub jvm: Option<Arc<Mutex<JavaVMPtr>>>,
    pub env: *mut sys::JNIEnv,
    pub methods: sys::JNINativeInterface_,
    thread_id: ThreadId,
    options: Option<InternalJavaOptions>,
    _marker: std::marker::PhantomData<&'a *mut sys::JNIEnv>,
}

impl<'a> JavaEnvWrapper<'a> {
    /// You should probably not use this.
    pub(crate) fn new(
        jvm: Arc<Mutex<JavaVMPtr>>,
        options: InternalJavaOptions,
        env: *mut sys::JNIEnv,
    ) -> Self {
        jvm.lock().unwrap().increase_ref_count();
        Self {
            jvm: Some(jvm),
            env,
            methods: unsafe { *(*env) },
            options: Some(options),
            thread_id: std::thread::current().id(),
            _marker: std::marker::PhantomData,
        }
    }

    pub unsafe fn from_raw(env: *mut sys::JNIEnv, options: InternalJavaOptions) -> Self {
        let mut res = Self {
            jvm: None,
            env,
            methods: *(*env),
            options: Some(options),
            thread_id: std::thread::current().id(),
            _marker: std::marker::PhantomData,
        };

        res.jvm = res.load_java_vm().ok();
        res
    }

    fn load_java_vm(&self) -> ResultType<Arc<Mutex<JavaVMPtr>>> {
        let mut vm: *mut sys::JavaVM = ptr::null_mut();
        let res =
            unsafe { self.methods.GetJavaVM.unwrap()(self.env, &mut vm as *mut *mut sys::JavaVM) };
        if res != sys::JNI_OK as _ || vm == ptr::null_mut() {
            Err(format!("Failed to get JavaVM: {}", jni_error_to_string(res)).into())
        } else {
            let options = self.options.ok_or("The options were unset".to_string())?;
            Ok(Arc::new(Mutex::new(JavaVMPtr::from_raw(vm, options))))
        }
    }

    pub fn get_object_class(&'a self, object: JavaObject) -> ResultType<JavaClass<'a>> {
        let class = unsafe {
            self.methods.GetObjectClass.unwrap()(
                self.env,
                object
                    .get_raw()
                    .ok_or("Cannot get class of null object".to_string())?,
            )
        };

        if self.is_err() {
            return Err(self.get_last_error(file!(), line!(), true, "GetObjectClass failed")?);
        }

        Ok(JavaClass::new(class, self))
    }

    pub fn get_object_signature(&self, object: JavaObject) -> ResultType<JavaType> {
        let object_class = self.get_object_class(object.clone())?;

        let get_class = object_class.get_object_method("getClass", "()Ljava/lang/Class;")?;
        let class = get_class.call(object, vec![])?;
        let java_class = self.get_java_lang_class()?;

        let get_name = java_class.get_object_method("getName", "()Ljava/lang/String;")?;
        let java_name = get_name.call(JavaObject::from(class), JavaArgs::new())?;
        let name = JavaString::from(java_name).to_string()?;

        Ok(JavaType::new(name, true))
    }

    pub fn is_instance_of(&self, object: JavaObject, classname: &str) -> ResultType<bool> {
        let class = self.find_class(classname, true)?;

        let result = unsafe {
            self.methods.IsInstanceOf.unwrap()(
                self.env,
                object.get_raw().ok_or(
                    "Cannot check if object is instance of class with null object".to_string(),
                )?,
                class.class()?,
            )
        };
        if self.is_err() {
            return Err(self.get_last_error(file!(), line!(), true, "IsInstanceOf failed")?);
        }

        Ok(result != 0)
    }

    pub fn throw_error(&self, message: String) {
        let message = CString::new(message).unwrap();
        unsafe {
            self.methods.ThrowNew.unwrap()(
                self.env,
                self.find_class("java/lang/Exception", true)
                    .unwrap()
                    .class()
                    .unwrap(),
                message.as_ptr(),
            );
        }
    }

    pub fn throw(&self, object: JavaObject) {
        unsafe {
            self.methods.Throw.unwrap()(self.env, object.get_raw().unwrap());
        }
    }

    /// Check if an error has been thrown inside this environment
    ///
    /// See also [`get_last_error`](Self::get_last_error).
    fn is_err(&self) -> bool {
        unsafe { self.methods.ExceptionCheck.unwrap()(self.env) != 0 }
    }

    /// Clear the (last) pending exception from this environment.
    ///
    /// See also [`get_last_error`](Self::get_last_error).
    fn clear_err(&self) {
        unsafe {
            self.methods.ExceptionClear.unwrap()(self.env);
        }
    }

    /// Get the pending exception from this environment.
    /// If no exception is pending, returns an error.
    ///
    /// See also [`get_last_error`](Self::get_last_error).
    fn exception_occurred(&'a self) -> ResultType<LocalJavaObject<'a>> {
        let throwable = unsafe { self.methods.ExceptionOccurred.unwrap()(self.env) };
        self.clear_err();
        if throwable == ptr::null_mut() {
            return Err(JNIError::from("Call to ExceptionOccurred failed").into());
        }

        Ok(LocalJavaObject::new(throwable, self))
    }

    /// Convert the frames of the last pending exception to a rust error.
    ///
    /// See [`get_last_error`](Self::get_last_error) which uses this method.
    fn convert_frames(
        &self,
        frames: &mut JavaObjectArray<'_>,
        num_frames: i32,
        throwable: &LocalJavaObject,
        throwable_to_string: &JavaObjectMethod,
        throwable_get_cause: &JavaObjectMethod,
        throwable_get_stack_trace: &JavaObjectMethod,
        stack_trace_element_to_string: &JavaObjectMethod,
        causes: &mut Vec<String>,
        stack_frames: &mut Vec<String>,
    ) -> ResultType<()> {
        if frames.is_null() {
            return Ok(());
        }

        let throwable_string =
            throwable_to_string.call_with_errors(JavaObject::from(throwable), vec![], false)?;
        causes.push(throwable_string.to_java_string().try_into()?);

        for i in 0..num_frames {
            let frame = frames.get_with_errors(i, false)?;
            let frame_string = stack_trace_element_to_string.call_with_errors(
                JavaObject::from(&frame),
                vec![],
                false,
            )?;
            stack_frames.push(frame_string.to_java_string().try_into()?);
        }

        let throwable =
            throwable_get_cause.call_with_errors(JavaObject::from(throwable), vec![], false)?;
        if throwable.is_null() {
            return Ok(());
        }

        let mut frames = JavaObjectArray::from(throwable_get_stack_trace.call_with_errors(
            JavaObject::from(&throwable),
            vec![],
            false,
        )?);
        let num_frames = frames.len()?;

        self.convert_frames(
            &mut frames,
            num_frames,
            &throwable,
            throwable_to_string,
            throwable_get_cause,
            throwable_get_stack_trace,
            stack_trace_element_to_string,
            causes,
            stack_frames,
        )
    }

    /// Get the last java error as an rust error.
    /// If no error is pending, returns an error.
    /// Clears the pending exception, converts the stack frames
    /// and returns the error as an [`JavaError`](crate::jni::java_error::JavaError).
    /// If this returns `Err`, an error occurred while converting the stack frames,
    /// if this returns `Ok`, everything was converted correctly.
    ///
    /// # Example
    /// ```rust
    /// // Only call this if you are sure there is an error pending.
    /// if self.is_err() {
    ///     return Err(self.get_last_error(file!(), line!(), true, "Alt text")?);
    /// }
    /// ```
    ///
    /// # Parameters
    /// - `file` - The (source) file the error occurred in.
    /// - `line` - The line the error occurred on.
    /// - `convert_errors` - For internal use only, just pass `true`.
    /// - `alt_text` - An alternative text to use when `convert_errors` is `false`.
    fn get_last_error(
        &self,
        file: &str,
        line: u32,
        convert_errors: bool,
        alt_text: &str,
    ) -> ResultType<Box<dyn Error>> {
        if !self.is_err() {
            return Err(JNIError::from("No error occurred").into());
        }

        let mut own_stack_frames = vec![
            format!("{}:{}", file, line).to_string(),
            format!("{}:{}", file!(), line!()).to_string(),
        ];

        let mut stack_frames: Vec<String> = Vec::new();

        if !convert_errors {
            self.clear_err();
            return Err(JavaError::new(vec![], own_stack_frames, alt_text.to_string()).into());
        }

        let throwable = self.exception_occurred()?;

        let throwable_class = self.find_class("java/lang/Throwable", false)?;
        let throwable_get_cause = throwable_class.get_object_method_with_errors(
            "getCause",
            "()Ljava/lang/Throwable;",
            false,
        )?;
        let throwable_get_stack_trace = throwable_class.get_object_method_with_errors(
            "getStackTrace",
            "()[Ljava/lang/StackTraceElement;",
            false,
        )?;
        let throwable_to_string = throwable_class.get_object_method_with_errors(
            "toString",
            "()Ljava/lang/String;",
            false,
        )?;

        let stack_trace_element_class = self.find_class("java/lang/StackTraceElement", false)?;
        let stack_trace_element_to_string = stack_trace_element_class
            .get_object_method_with_errors("toString", "()Ljava/lang/String;", false)?;

        let mut frames = JavaObjectArray::from(throwable_get_stack_trace.call_with_errors(
            JavaObject::from(&throwable),
            vec![],
            false,
        )?);
        let num_frames = frames.len()?;

        let mut causes: Vec<String> = vec![];

        self.convert_frames(
            &mut frames,
            num_frames,
            &throwable,
            &throwable_to_string,
            &throwable_get_cause,
            &throwable_get_stack_trace,
            &stack_trace_element_to_string,
            &mut causes,
            &mut stack_frames,
        )?;
        stack_frames.append(&mut own_stack_frames);
        Ok(JavaError::new(causes, stack_frames, alt_text.to_string()).into())
    }

    /// Delete the local reference of an java object.
    /// Make sure to only call this once as any subsequent
    /// calls will cause the program to segfault.
    /// This is not strictly required by the jvm but should be done
    /// for jni calls to the jvm. Do not call this if the object
    /// has been converted to a global reference.
    pub fn delete_local_ref(&self, object: sys::jobject) -> () {
        unsafe {
            self.methods.DeleteLocalRef.unwrap()(self.env, object);
        }
    }

    /// Create a new global reference to a java object.
    ///
    /// Used by [`GlobalJavaObject`](crate::jni::java_object::GlobalJavaObject)
    /// to create a global references from local ones.
    pub fn new_global_object(&self, object: sys::jobject) -> ResultType<GlobalJavaObject> {
        unsafe {
            let obj = self.methods.NewGlobalRef.unwrap()(self.env, object);
            if self.is_err() {
                self.clear_err();
                return Err(JNIError::new("Failed to create global reference".to_string()).into());
            }

            Ok(GlobalJavaObject::new(
                obj,
                self.jvm
                    .as_ref()
                    .ok_or("The jvm was unset".to_string())?
                    .clone(),
                self.options
                    .ok_or("The options were unset".to_string())?
                    .clone(),
            ))
        }
    }

    /// Find a class by its jni class name.
    ///
    /// Used by [`JavaEnv.find_class()`](crate::jni::java_env::JavaEnv::find_class)
    /// and [`JavaClass.by_name()`](crate::jni::java_object::JavaClass::by_name).
    pub fn find_class(
        &'a self,
        class_name: &str,
        resolve_errors: bool,
    ) -> ResultType<JavaClass<'a>> {
        let c_class_name = CString::new(class_name)?;
        unsafe {
            let class = self.methods.FindClass.unwrap()(self.env, c_class_name.as_ptr());
            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    resolve_errors,
                    format!("Could not find class '{}'", class_name).as_str(),
                )?);
            }

            Ok(JavaClass::new(class, &self))
        }
    }

    /// Get `java.lang.Class`
    pub fn get_java_lang_class(&'a self) -> ResultType<JavaClass<'a>> {
        self.find_class("java/lang/Class", true)
    }

    pub fn find_class_by_java_name(&'a self, class_name: String) -> ResultType<JavaClass<'a>> {
        let class = self.get_java_lang_class()?;
        let for_name = class.get_static_object_method(
            "forName",
            "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;",
        )?;
        let java_class_name = JavaString::_try_from(class_name, self)?;
        let arg = JavaBoolean::new(true);

        let loader = self
            .jvm
            .as_ref()
            .ok_or("The jvm was unset".to_string())?
            .lock()
            .unwrap()
            .class_loader()
            .clone()
            .unwrap();
        let class_loader = LocalJavaObject::from_internal(loader.borrow(), self);
        let res = for_name.call(vec![
            Box::new(&java_class_name),
            Box::new(&arg),
            Box::new(&class_loader),
        ])?;

        Ok(JavaClass::from(res.assign_env(self)))
    }

    /// Find a class by its java class name
    ///
    /// Used by [`JavaEnv.find_class_by_java_name()`](crate::jni::java_env::JavaEnv::find_class_by_java_name)
    /// and [`GlobalJavaClass.by_name()`](crate::jni::java_object::GlobalJavaClass::by_name).
    pub fn find_global_class_by_java_name(
        &'a self,
        class_name: String,
    ) -> ResultType<GlobalJavaClass> {
        let class = self.get_java_lang_class()?;
        let for_name = class.get_static_object_method(
            "forName",
            "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;",
        )?;
        let java_class_name = JavaString::_try_from(class_name, self)?;
        let arg = JavaBoolean::new(true);

        let loader = self
            .jvm
            .as_ref()
            .ok_or("The jvm was unset".to_string())?
            .lock()
            .unwrap()
            .class_loader()
            .clone()
            .unwrap();
        let class_loader = LocalJavaObject::from_internal(loader.borrow(), self);
        let cls = GlobalJavaClass::try_from(for_name.call(vec![
            Box::new(&java_class_name),
            Box::new(&arg),
            Box::new(&class_loader),
        ])?)?;
        Ok(cls)
    }

    pub fn get_system_class_loader(&self) -> ResultType<GlobalJavaObject> {
        let class = self.find_class("java/lang/ClassLoader", true)?;
        let get_system_class_loader =
            class.get_static_object_method("getSystemClassLoader", "()Ljava/lang/ClassLoader;")?;

        let loader = get_system_class_loader.call(vec![])?;
        GlobalJavaObject::try_from(loader)
    }

    pub fn get_method_id(
        &self,
        class: &'a JavaClass<'a>,
        method_name: &str,
        signature: &str,
    ) -> ResultType<JavaMethod<'a>> {
        self.get_method_id_with_errors(class, method_name, signature, true)
    }

    pub fn get_method_id_with_errors(
        &self,
        class: &'a JavaClass<'a>,
        method_name: &str,
        signature: &str,
        resolve_errors: bool,
    ) -> ResultType<JavaMethod<'a>> {
        let method_name_str = CString::new(method_name)?;
        let signature_str = CString::new(signature)?;
        unsafe {
            let method = self.methods.GetMethodID.unwrap()(
                self.env,
                class.class()?,
                method_name_str.as_ptr(),
                signature_str.as_ptr(),
            );

            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    resolve_errors,
                    format!("Could not find method '{}{}'", method_name, signature).as_str(),
                )?);
            }

            Ok(JavaMethod::new(
                method,
                class,
                JavaType::from_method_return_type(signature)?,
                false,
            ))
        }
    }

    pub fn get_static_method_id(
        &self,
        class: &'a JavaClass<'a>,
        method_name: &str,
        signature: &str,
    ) -> ResultType<JavaMethod<'a>> {
        let method_name_str = CString::new(method_name)?;
        let signature_str = CString::new(signature)?;
        unsafe {
            let method = self.methods.GetStaticMethodID.unwrap()(
                self.env,
                class.class()?,
                method_name_str.as_ptr(),
                signature_str.as_ptr(),
            );

            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    true,
                    format!("Could not find method '{}'", method_name).as_str(),
                )?);
            }

            Ok(JavaMethod::new(
                method,
                class,
                JavaType::from_method_return_type(signature)?,
                true,
            ))
        }
    }

    unsafe fn convert_args(&self, args: JavaArgs) -> Vec<sys::jvalue> {
        args.iter().map(|arg| arg.to_java_value().value()).collect()
    }

    pub fn call_object_method(
        &'a self,
        object: JavaObject<'_>,
        method: &JavaMethod<'_>,
        args: JavaArgs<'_>,
    ) -> ResultType<LocalJavaObject<'a>> {
        self.call_object_method_with_errors(object, method, args, true)
    }

    pub fn call_object_method_with_errors(
        &'a self,
        object: JavaObject<'_>,
        method: &JavaMethod<'_>,
        args: JavaArgs<'_>,
        resolve_errors: bool,
    ) -> ResultType<LocalJavaObject<'a>> {
        unsafe {
            let args = self.convert_args(args);
            let obj = self.methods.CallObjectMethodA.unwrap()(
                self.env,
                object
                    .get_raw()
                    .ok_or("Cannot call object method with null object".to_string())?,
                method.id(),
                args.as_ptr(),
            );
            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    resolve_errors,
                    "CallObjectMethodA failed",
                )?);
            }

            Ok(LocalJavaObject::new(obj, self))
        }
    }

    pub fn call_static_object_method(
        &'a self,
        class: &JavaClass<'_>,
        method: &JavaMethod<'_>,
        args: JavaArgs<'_>,
    ) -> ResultType<LocalJavaObject<'a>> {
        unsafe {
            let args = self.convert_args(args);
            let obj = self.methods.CallStaticObjectMethodA.unwrap()(
                self.env,
                class.class()?,
                method.id(),
                args.as_ptr(),
            );
            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    true,
                    format!("CallStaticObjectMethod failed").as_str(),
                )?);
            }

            Ok(LocalJavaObject::new(obj, self))
        }
    }

    define_call_methods!(
        call_int_method,
        call_static_int_method,
        CallIntMethodA,
        CallStaticIntMethodA,
        i32,
        r,
        r
    );

    define_call_methods!(
        call_long_method,
        call_static_long_method,
        CallLongMethodA,
        CallStaticLongMethodA,
        i64,
        r,
        r
    );

    define_call_methods!(
        call_float_method,
        call_static_float_method,
        CallFloatMethodA,
        CallStaticFloatMethodA,
        f32,
        r,
        r
    );

    define_call_methods!(
        call_boolean_method,
        call_static_boolean_method,
        CallBooleanMethodA,
        CallStaticBooleanMethodA,
        bool,
        r,
        r != 0
    );

    define_call_methods!(
        call_byte_method,
        call_static_byte_method,
        CallByteMethodA,
        CallStaticByteMethodA,
        i8,
        r,
        r
    );

    define_call_methods!(
        call_char_method,
        call_static_char_method,
        CallCharMethodA,
        CallStaticCharMethodA,
        u16,
        r,
        r
    );

    define_call_methods!(
        call_short_method,
        call_static_short_method,
        CallShortMethodA,
        CallStaticShortMethodA,
        i16,
        r,
        r
    );

    define_call_methods!(
        call_double_method,
        call_static_double_method,
        CallDoubleMethodA,
        CallStaticDoubleMethodA,
        f64,
        r,
        r
    );

    define_call_methods!(
        call_void_method,
        call_static_void_method,
        CallVoidMethodA,
        CallStaticVoidMethodA,
        (),
        r,
        r
    );

    pub fn get_field_id(
        &'a self,
        class: &'a JavaClass<'a>,
        name: String,
        signature: JavaType,
        is_static: bool,
    ) -> ResultType<JavaField<'a>> {
        let field_name = CString::new(name.clone())?;
        let field_signature = CString::new(signature.to_jni_type())?;
        unsafe {
            let field = if is_static {
                self.methods.GetStaticFieldID.unwrap()(
                    self.env,
                    class.class()?,
                    field_name.as_ptr(),
                    field_signature.as_ptr(),
                )
            } else {
                self.methods.GetFieldID.unwrap()(
                    self.env,
                    class.class()?,
                    field_name.as_ptr(),
                    field_signature.as_ptr(),
                )
            };

            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    true,
                    format!("Could not find field '{}'", name).as_str(),
                )?);
            }

            Ok(JavaField::new(field, signature, class, is_static))
        }
    }

    pub fn get_object_field(
        &'a self,
        field: &JavaObjectField<'_>,
        object: &JavaObject<'_>,
    ) -> ResultType<JavaObject<'a>> {
        unsafe {
            let res = self.methods.GetObjectField.unwrap()(
                self.env,
                object
                    .get_raw()
                    .ok_or("Cannot get field of null object".to_string())?,
                field.id(),
            );
            if self.is_err() {
                Err(self.get_last_error(file!(), line!(), true, "GetObjectField failed")?)
            } else {
                Ok(JavaObject::from(LocalJavaObject::new(res, self)))
            }
        }
    }

    pub fn set_object_field(
        &'a self,
        field: &JavaObjectField<'_>,
        object: &JavaObject<'_>,
        value: JavaObject<'_>,
    ) -> ResultType<()> {
        unsafe {
            self.methods.SetObjectField.unwrap()(
                self.env,
                object
                    .get_raw()
                    .ok_or("Cannot set field of null object".to_string())?,
                field.id(),
                value.get_raw_nullable(),
            );
            if self.is_err() {
                Err(self.get_last_error(file!(), line!(), true, "SetObjectField failed")?)
            } else {
                Ok(())
            }
        }
    }

    pub fn get_static_object_field(
        &'a self,
        field: &StaticJavaObjectField,
        class: &JavaClass<'_>,
    ) -> ResultType<JavaObject<'a>> {
        unsafe {
            let res =
                self.methods.GetStaticObjectField.unwrap()(self.env, class.class()?, field.id());
            if self.is_err() {
                Err(self.get_last_error(file!(), line!(), true, "GetStaticObjectField failed")?)
            } else {
                Ok(JavaObject::from(LocalJavaObject::new(res, self)))
            }
        }
    }

    pub fn set_static_object_field(
        &'a self,
        field: &StaticJavaObjectField,
        class: &JavaClass<'_>,
        value: JavaObject<'_>,
    ) -> ResultType<()> {
        unsafe {
            self.methods.SetStaticObjectField.unwrap()(
                self.env,
                class.class()?,
                field.id(),
                value.get_raw_nullable(),
            );
            if self.is_err() {
                Err(self.get_last_error(
                    file!(),
                    line!(),
                    true,
                    concat!(stringify!($static_setter), " failed"),
                )?)
            } else {
                Ok(())
            }
        }
    }

    define_field_methods!(
        get_int_field,
        set_int_field,
        get_static_int_field,
        set_static_int_field,
        JavaIntField,
        StaticJavaIntField,
        i32,
        GetIntField,
        SetIntField,
        GetStaticIntField,
        SetStaticIntField
    );
    define_field_methods!(
        get_long_field,
        set_long_field,
        get_static_long_field,
        set_static_long_field,
        JavaLongField,
        StaticJavaLongField,
        i64,
        GetLongField,
        SetLongField,
        GetStaticLongField,
        SetStaticLongField
    );
    define_field_methods!(
        get_float_field,
        set_float_field,
        get_static_float_field,
        set_static_float_field,
        JavaFloatField,
        StaticJavaFloatField,
        f32,
        GetFloatField,
        SetFloatField,
        GetStaticFloatField,
        SetStaticFloatField
    );
    define_field_methods!(
        get_boolean_field,
        set_boolean_field,
        get_static_boolean_field,
        set_static_boolean_field,
        JavaBooleanField,
        StaticJavaBooleanField,
        u8,
        GetBooleanField,
        SetBooleanField,
        GetStaticBooleanField,
        SetStaticBooleanField
    );
    define_field_methods!(
        get_byte_field,
        set_byte_field,
        get_static_byte_field,
        set_static_byte_field,
        JavaByteField,
        StaticJavaByteField,
        i8,
        GetByteField,
        SetByteField,
        GetStaticByteField,
        SetStaticByteField
    );
    define_field_methods!(
        get_char_field,
        set_char_field,
        get_static_char_field,
        set_static_char_field,
        JavaCharField,
        StaticJavaCharField,
        u16,
        GetCharField,
        SetCharField,
        GetStaticCharField,
        SetStaticCharField
    );
    define_field_methods!(
        get_short_field,
        set_short_field,
        get_static_short_field,
        set_static_short_field,
        JavaShortField,
        StaticJavaShortField,
        i16,
        GetShortField,
        SetShortField,
        GetStaticShortField,
        SetStaticShortField
    );
    define_field_methods!(
        get_double_field,
        set_double_field,
        get_static_double_field,
        set_static_double_field,
        JavaDoubleField,
        StaticJavaDoubleField,
        f64,
        GetDoubleField,
        SetDoubleField,
        GetStaticDoubleField,
        SetStaticDoubleField
    );

    pub fn get_object_array_element(
        &'a self,
        array: &'a JavaArray<'a>,
        index: i32,
        resolve_errors: bool,
    ) -> ResultType<LocalJavaObject<'a>> {
        unsafe {
            let obj = self.methods.GetObjectArrayElement.unwrap()(
                self.env,
                array
                    .get_raw()
                    .ok_or("Cannot get array element of null array".to_string())?,
                index,
            );
            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    resolve_errors,
                    "GetObjectArrayElement failed",
                )?);
            }

            Ok(LocalJavaObject::new(obj, self))
        }
    }

    pub fn set_object_array_element(
        &'a self,
        array: &'a JavaArray<'a>,
        index: i32,
        element: JavaObject<'a>,
    ) -> ResultType<()> {
        unsafe {
            self.methods.SetObjectArrayElement.unwrap()(
                self.env,
                array
                    .get_raw()
                    .ok_or("Cannot set element of null array".to_string())?,
                index,
                element.get_raw_nullable(),
            );
            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    true,
                    "SetObjectArrayElement failed",
                )?);
            }

            Ok(())
        }
    }

    pub fn get_array_length(&self, array: sys::jobject) -> ResultType<i32> {
        unsafe {
            let length = self.methods.GetArrayLength.unwrap()(self.env, array);
            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    true,
                    "GetArrayLength failed",
                )?);
            }

            Ok(length)
        }
    }

    pub fn create_object_array(
        &self,
        class: &'a JavaClass<'a>,
        len: i32,
    ) -> ResultType<JavaObjectArray> {
        unsafe {
            let arr = self.methods.NewObjectArray.unwrap()(
                self.env,
                len,
                class.class()?,
                ptr::null_mut(),
            );
            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    true,
                    "NewObjectArray failed",
                )?);
            }

            Ok(JavaObjectArray::from(LocalJavaObject::new(arr, self)))
        }
    }

    define_array_methods!(
        create_short_array,
        get_short_array_elements,
        *const i16,
        i16,
        JavaShortArray,
        NewShortArray,
        SetShortArrayRegion,
        GetShortArrayElements,
        ReleaseShortArrayElements
    );
    define_array_methods!(
        create_int_array,
        get_int_array_elements,
        *const i32,
        i32,
        JavaIntArray,
        NewIntArray,
        SetIntArrayRegion,
        GetIntArrayElements,
        ReleaseIntArrayElements
    );
    define_array_methods!(
        create_long_array,
        get_long_array_elements,
        *const i64,
        i64,
        JavaLongArray,
        NewLongArray,
        SetLongArrayRegion,
        GetLongArrayElements,
        ReleaseLongArrayElements
    );
    define_array_methods!(
        create_float_array,
        get_float_array_elements,
        *const f32,
        f32,
        JavaFloatArray,
        NewFloatArray,
        SetFloatArrayRegion,
        GetFloatArrayElements,
        ReleaseFloatArrayElements
    );
    define_array_methods!(
        create_double_array,
        get_double_array_elements,
        *const f64,
        f64,
        JavaDoubleArray,
        NewDoubleArray,
        SetDoubleArrayRegion,
        GetDoubleArrayElements,
        ReleaseDoubleArrayElements
    );
    define_array_methods!(
        create_boolean_array,
        get_boolean_array_elements,
        *const u8,
        u8,
        JavaBooleanArray,
        NewBooleanArray,
        SetBooleanArrayRegion,
        GetBooleanArrayElements,
        ReleaseBooleanArrayElements
    );
    define_array_methods!(
        create_byte_array,
        get_byte_array_elements,
        *const i8,
        i8,
        JavaByteArray,
        NewByteArray,
        SetByteArrayRegion,
        GetByteArrayElements,
        ReleaseByteArrayElements
    );
    define_array_methods!(
        create_char_array,
        get_char_array_elements,
        *const u16,
        u16,
        JavaCharArray,
        NewCharArray,
        SetCharArrayRegion,
        GetCharArrayElements,
        ReleaseCharArrayElements
    );

    pub unsafe fn get_string_utf_chars(&self, string: sys::jstring) -> ResultType<String> {
        let chars = self.methods.GetStringUTFChars.unwrap()(self.env, string, ptr::null_mut());
        if self.is_err() || chars == ptr::null_mut() {
            self.clear_err();
            return Err(JNIError::from("GetStringUTFChars failed").into());
        }

        let c_string = CStr::from_ptr(chars);
        let converted = c_string.to_str().map(|s| s.to_string());
        self.methods.ReleaseStringUTFChars.unwrap()(self.env, string, chars);
        if self.is_err() {
            self.clear_err();
            return Err(JNIError::from("ReleaseStringUTFChars failed").into());
        }

        Ok(converted?)
    }

    pub fn string_to_java_string(&'a self, string: String) -> ResultType<JavaString<'a>> {
        let c_string = CString::new(string)?;
        unsafe {
            let string = self.methods.NewStringUTF.unwrap()(self.env, c_string.as_ptr());
            if self.is_err() || string == ptr::null_mut() {
                self.clear_err();
                return Err(JNIError::from("NewStringUTF failed").into());
            }

            Ok(JavaString::new(self, string))
        }
    }

    pub(in crate::jni) fn get_java_vm(&self) -> ResultType<JavaVM> {
        Ok(JavaVM::from_existing(
            self.jvm
                .as_ref()
                .ok_or("jvm was unset".to_string())?
                .clone(),
            self.options.ok_or("The options were unset".to_string())?,
        ))
    }

    pub unsafe fn is_assignable_from(
        &self,
        sub: sys::jclass,
        sup: sys::jclass,
    ) -> ResultType<bool> {
        let result = self.methods.IsAssignableFrom.unwrap()(self.env, sub, sup);
        if self.is_err() {
            Err(self.get_last_error(file!(), line!(), true, "IsAssignableFrom failed")?)
        } else {
            Ok(result != 0)
        }
    }

    pub fn new_instance(
        &'a self,
        constructor: &JavaConstructor,
        args: JavaArgs,
    ) -> ResultType<LocalJavaObject<'a>> {
        let res = unsafe {
            let args = self.convert_args(args);
            self.methods.NewObjectA.unwrap()(
                self.env,
                constructor.class()?,
                constructor.id(),
                args.as_ptr(),
            )
        };

        if self.is_err() || res == ptr::null_mut() {
            Err(self.get_last_error(file!(), line!(), true, "NewObjectA failed")?)
        } else {
            Ok(LocalJavaObject::new(res, self))
        }
    }

    pub fn get_constructor(
        &'a self,
        class: &'a JavaClass<'a>,
        signature: &str,
    ) -> ResultType<JavaConstructor<'a>> {
        let id = unsafe {
            self.methods.GetMethodID.unwrap()(
                self.env,
                class.class()?,
                CString::new("<init>")?.as_ptr(),
                CString::new(signature)?.as_ptr(),
            )
        };

        if self.is_err() || id == ptr::null_mut() {
            Err(self.get_last_error(file!(), line!(), true, "GetMethodID failed")?)
        } else {
            Ok(JavaConstructor::new(id, class))
        }
    }

    pub fn append_class_path(&self, paths: Vec<String>) -> ResultType<()> {
        let file_class = self.find_class("java/io/File", true)?;
        let file_constructor = file_class.get_constructor("(Ljava/lang/String;)V")?;
        let to_uri = file_class.get_object_method("toURI", "()Ljava/net/URI;")?;

        let uri_class = self.find_class("java/net/URI", true)?;
        let to_url = uri_class.get_object_method("toURL", "()Ljava/net/URL;")?;

        let url_class_loader = self.find_class("java/net/URLClassLoader", true)?;
        let class_loader_constructor =
            url_class_loader.get_constructor("([Ljava/net/URL;Ljava/lang/ClassLoader;)V")?;

        let url_class = self.find_class("java/net/URL", true)?;
        let mut urls = self.create_object_array(&url_class, 1)?;
        for i in 0..paths.len() {
            let java_path = self.string_to_java_string(paths.get(i).unwrap().clone())?;
            let file = self.new_instance(&file_constructor, vec![Box::new(&java_path)])?;

            let uri = to_uri.call(JavaObject::from(file), vec![])?;
            let url = to_url.call(JavaObject::from(uri), vec![])?;

            urls.set(i as i32, JavaObject::from(url))?;
        }

        let old_class_loader = self
            .jvm
            .as_ref()
            .ok_or("The jvm was unset".to_string())?
            .lock()
            .unwrap()
            .class_loader()
            .as_ref()
            .unwrap()
            .clone();

        let class_loader = self.new_instance(
            &class_loader_constructor,
            vec![Box::new(&urls), Box::new(&old_class_loader)],
        )?;

        let res = GlobalJavaObject::try_from(class_loader)?;
        self.jvm
            .as_ref()
            .ok_or("The jvm was unset".to_string())?
            .lock()
            .unwrap()
            .set_class_loader(res);

        Ok(())
    }

    pub fn get_vm_ptr(&self) -> ResultType<Arc<Mutex<JavaVMPtr>>> {
        Ok(self
            .jvm
            .as_ref()
            .ok_or("The jvm was unset".to_string())?
            .clone())
    }

    pub fn get_options(&self) -> ResultType<InternalJavaOptions> {
        self.options.ok_or("The options were unset".into())
    }
}

impl<'a> Drop for JavaEnvWrapper<'a> {
    fn drop(&mut self) {
        if self.jvm.is_none() {
            return;
        }

        if std::thread::current().id() != self.thread_id {
            panic!("JavaEnvWrapper was dropped from a different thread than it was created on, {:?} vs. {:?}", std::thread::current().id(), self.thread_id);
        }

        self.jvm
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .decrease_ref_count();
    }
}
