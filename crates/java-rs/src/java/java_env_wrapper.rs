use crate::java::java_error::JavaError;
use crate::java::java_field::{
    JavaBooleanField, JavaByteField, JavaCharField, JavaDoubleField, JavaField, JavaFloatField,
    JavaIntField, JavaLongField, JavaObjectField, JavaShortField, StaticJavaBooleanField,
    StaticJavaByteField, StaticJavaCharField, StaticJavaDoubleField, StaticJavaFloatField,
    StaticJavaIntField, StaticJavaLongField, StaticJavaObjectField, StaticJavaShortField,
};
use crate::java::java_type::JavaType;
use crate::java::java_vm::JavaVM;
use crate::java::jni_error::JNIError;
use crate::java::objects::args::JavaArgs;
use crate::java::objects::array::{
    JavaArray, JavaBooleanArray, JavaByteArray, JavaCharArray, JavaDoubleArray, JavaFloatArray,
    JavaIntArray, JavaLongArray, JavaObjectArray, JavaShortArray,
};
use crate::java::objects::class::{GlobalJavaClass, JavaClass};
use crate::java::objects::constructor::JavaConstructor;
use crate::java::objects::java_object::JavaObject;
use crate::java::objects::method::{JavaMethod, JavaObjectMethod};
use crate::java::objects::object::{GlobalJavaObject, LocalJavaObject};
use crate::java::objects::string::JavaString;
use crate::java::objects::value::JavaBoolean;
use crate::java::traits::GetRaw;
use crate::java::util::util::{jni_error_to_string, ResultType};
use crate::java::vm_ptr::JavaVMPtr;
use crate::objects::args::AsJavaArg;
#[cfg(feature = "type_check")]
use crate::signature::Signature;
#[cfg(feature = "type_check")]
use crate::traits::GetSignature;
use crate::traits::GetSignatureRef;
use crate::{
    assert_non_null, define_array_methods, define_call_methods, define_field_methods, sys,
};
use std::borrow::Borrow;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::ops::Not;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

/// A wrapper around a JNIEnv.
/// This manages the reference count for the current thread
/// and provides convenience methods to the jvm.
/// Rather than copying or creating this directly, attach a new
/// thread using [`JavaVM::attach_thread`](JavaVM::attach_thread).
pub struct JavaEnvWrapper<'a> {
    pub jvm: Option<Arc<Mutex<JavaVMPtr>>>,
    pub env: *mut sys::JNIEnv,
    pub methods: sys::JNINativeInterface_,
    thread_id: ThreadId,
    _marker: std::marker::PhantomData<&'a *mut sys::JNIEnv>,
}

impl<'a> JavaEnvWrapper<'a> {
    /// You should probably not use this.
    pub(crate) fn new(jvm: Arc<Mutex<JavaVMPtr>>, env: *mut sys::JNIEnv) -> Self {
        #[cfg(feature = "log")]
        crate::trace!(
            "Creating JavaEnv in thread {:?}",
            std::thread::current().id()
        );

        assert_non_null!(env, "JavaEnvWrapper::new: env is null");
        Self {
            jvm: Some(jvm),
            env,
            methods: unsafe { *(*env) },
            thread_id: std::thread::current().id(),
            _marker: std::marker::PhantomData,
        }
    }

    pub unsafe fn from_raw(env: *mut sys::JNIEnv) -> Self {
        assert_non_null!(env, "JavaEnvWrapper::from_raw: env is null");
        let mut res = Self {
            jvm: None,
            env,
            methods: *(*env),
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

        if res != sys::JNI_OK as _ || vm.is_null() {
            Err(format!("Failed to get JavaVM: {}", jni_error_to_string(res)).into())
        } else {
            Ok(Arc::new(Mutex::new(JavaVMPtr::from_raw(vm))))
        }
    }

    pub fn get_object_class(&'a self, object: JavaObject) -> ResultType<JavaClass<'a>> {
        let class = unsafe { self.methods.GetObjectClass.unwrap()(self.env, object.get_raw()) };

        if self.is_err() || class.is_null() {
            Err(self.get_last_error(file!(), line!(), true, "GetObjectClass failed")?)
        } else {
            Ok(JavaClass::new(
                class,
                self,
                #[cfg(feature = "type_check")]
                object.get_signature().clone(),
            ))
        }
    }

    pub fn get_object_signature(&self, object: JavaObject) -> ResultType<JavaType> {
        let object_class = self.get_object_class(object.clone())?;

        let get_class = object_class.get_object_method("getClass", "()Ljava/lang/Class;")?;
        let class = get_class
            .call(object, &[])?
            .ok_or("Object.getClass() returned null".to_string())?;
        let java_class = self.get_java_lang_class()?;

        let get_name = java_class.get_object_method("getName", "()Ljava/lang/String;")?;
        let java_name = get_name
            .call(JavaObject::from(class), &[])?
            .ok_or("Class.getName() returned null".to_string())?;

        // As JavaString::try_from calls get_object_signature in order
        // to check if the passed type is a string, we need to get the
        // string manually in order to prevent a stack overflow.
        let name = unsafe { self.get_string_utf_chars(java_name.get_raw())? };

        Ok(JavaType::new(name, true))
    }

    pub fn is_instance_of(&self, object: JavaObject, classname: &str) -> ResultType<bool> {
        let class = self.find_class(classname, true)?;

        let result = unsafe {
            self.methods.IsInstanceOf.unwrap()(self.env, object.get_raw(), class.class())
        };

        if self.is_err() {
            Err(self.get_last_error(file!(), line!(), true, "IsInstanceOf failed")?)
        } else {
            Ok(result != 0)
        }
    }

    pub fn instance_of(&self, this: JavaObject, other: GlobalJavaClass) -> ResultType<bool> {
        let result =
            unsafe { self.methods.IsInstanceOf.unwrap()(self.env, this.get_raw(), other.class()) };

        if self.is_err() {
            Err(self.get_last_error(file!(), line!(), true, "IsInstanceOf failed")?)
        } else {
            Ok(result != 0)
        }
    }

    pub fn throw_error(&self, message: String) {
        let message = CString::new(message).unwrap();
        unsafe {
            self.methods.ThrowNew.unwrap()(
                self.env,
                self.find_class("java/lang/Exception", true)
                    .unwrap()
                    .class(),
                message.as_ptr(),
            );
        }
    }

    pub fn throw(&self, object: JavaObject) {
        unsafe {
            self.methods.Throw.unwrap()(self.env, object.get_raw());
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
            Err(JNIError::from("Call to ExceptionOccurred failed").into())
        } else {
            Ok(LocalJavaObject::new(
                throwable,
                self,
                #[cfg(feature = "type_check")]
                JavaType::object(),
            ))
        }
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
        let throwable_string = throwable_to_string
            .call_with_errors(JavaObject::from(throwable), &[], false)?
            .ok_or("Throwable.toString() returned null".to_string())?;
        causes.push(throwable_string.to_java_string()?.try_into()?);

        for i in 0..num_frames {
            let frame = frames
                .get_with_errors(i, false)?
                .ok_or("A stack frame was null".to_string())?;
            let frame_string = stack_trace_element_to_string
                .call_with_errors(JavaObject::from(&frame), &[], false)?
                .ok_or("StackTraceElement.toString() returned null".to_string())?;
            stack_frames.push(frame_string.to_java_string()?.try_into()?);
        }

        let throwable =
            throwable_get_cause.call_with_errors(JavaObject::from(throwable), &[], false)?;
        let throwable = if let Some(throwable) = throwable {
            throwable
        } else {
            return Ok(());
        };

        let frames_obj =
            throwable_get_stack_trace.call_with_errors(JavaObject::from(&throwable), &[], false)?;

        let mut frames = if let Some(f) = frames_obj {
            JavaObjectArray::from(f)
        } else {
            return Ok(());
        };

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
    /// and returns the error as an [`JavaError`](JavaError).
    /// If this returns `Err`, an error occurred while converting the stack frames,
    /// if this returns `Ok`, everything was converted correctly.
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

        let mut frames = JavaObjectArray::from(
            throwable_get_stack_trace
                .call_with_errors(JavaObject::from(&throwable), &[], false)?
                .ok_or("Throwable.getStackTrace() returned null".to_string())?,
        );
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
        assert_non_null!(object);
        unsafe {
            self.methods.DeleteLocalRef.unwrap()(self.env, object);
        }
    }

    /// Create a new global reference to a java object.
    ///
    /// Used by [`GlobalJavaObject`](crate::java::java_object::GlobalJavaObject)
    /// to create a global references from local ones.
    pub fn new_global_object(
        &self,
        object: sys::jobject,
        #[cfg(feature = "type_check")] signature: JavaType,
    ) -> ResultType<GlobalJavaObject> {
        #[cfg(feature = "type_check")]
        crate::trace!("Creating global reference to object of type {}", signature);

        assert_non_null!(object);
        let obj = unsafe { self.methods.NewGlobalRef.unwrap()(self.env, object) };

        if self.is_err() || obj.is_null() {
            self.clear_err();
            Err(JNIError::new("Failed to create global reference".to_string()).into())
        } else {
            Ok(GlobalJavaObject::new(
                obj,
                self.jvm
                    .as_ref()
                    .ok_or("The jvm was unset".to_string())?
                    .clone(),
                #[cfg(feature = "type_check")]
                signature,
            ))
        }
    }

    /// Find a class by its jni class name.
    ///
    /// Used by [`JavaEnv.find_class()`](crate::java::java_env::JavaEnv::find_class)
    /// and [`JavaClass.by_name()`](crate::java::java_object::JavaClass::by_name).
    pub fn find_class(
        &'a self,
        class_name: &str,
        resolve_errors: bool,
    ) -> ResultType<JavaClass<'a>> {
        crate::trace!("Resolving class '{}'", class_name);
        let c_class_name = CString::new(class_name)?;

        let class = unsafe { self.methods.FindClass.unwrap()(self.env, c_class_name.as_ptr()) };

        if self.is_err() || class.is_null() {
            Err(self.get_last_error(
                file!(),
                line!(),
                resolve_errors,
                format!("Could not find class '{}'", class_name).as_str(),
            )?)
        } else {
            Ok(JavaClass::new(
                class,
                &self,
                #[cfg(feature = "type_check")]
                JavaType::new(class_name.to_string(), true),
            ))
        }
    }

    /// Get `java.lang.Class`
    pub fn get_java_lang_class(&'a self) -> ResultType<JavaClass<'a>> {
        crate::trace!("Getting java.lang.Class");
        self.find_class("java/lang/Class", true)
    }

    pub fn find_class_by_java_name(&'a self, class_name: String) -> ResultType<JavaClass<'a>> {
        crate::trace!("Resolving class '{}'", class_name);
        let class = self.get_java_lang_class()?;
        let for_name = class.get_static_object_method(
            "forName",
            "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;",
        )?;
        let java_class_name = JavaString::_try_from(class_name.clone(), self)?;
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
        let res = for_name
            .call(&[
                java_class_name.as_arg(),
                arg.as_arg(),
                class_loader.as_arg(),
            ])?
            .ok_or("Class.forName() returned null".to_string())?;

        Ok(JavaClass::from_local(
            res.assign_env(self),
            #[cfg(feature = "type_check")]
            JavaType::new(class_name, true),
        ))
    }

    /// Find a class by its java class name
    ///
    /// Used by [`JavaEnv.find_class_by_java_name()`](crate::java::java_env::JavaEnv::find_class_by_java_name)
    /// and [`GlobalJavaClass.by_name()`](crate::java::java_object::GlobalJavaClass::by_name).
    pub fn find_global_class_by_java_name(
        &'a self,
        class_name: String,
    ) -> ResultType<GlobalJavaClass> {
        crate::trace!("Resolving class '{}'", class_name);
        let class = self.get_java_lang_class()?;
        let for_name = class.get_static_object_method(
            "forName",
            "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;",
        )?;
        let java_class_name = JavaString::_try_from(class_name.clone(), self)?;
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
        let cls = GlobalJavaClass::try_from_local(
            for_name
                .call(&[
                    java_class_name.as_arg(),
                    arg.as_arg(),
                    class_loader.as_arg(),
                ])?
                .ok_or("Class.forName() returned null".to_string())?,
            #[cfg(feature = "type_check")]
            JavaType::new(class_name, true),
        )?;
        Ok(cls)
    }

    pub fn get_system_class_loader(&self) -> ResultType<GlobalJavaObject> {
        let class = self.find_class("java/lang/ClassLoader", true)?;
        let get_system_class_loader =
            class.get_static_object_method("getSystemClassLoader", "()Ljava/lang/ClassLoader;")?;

        let loader = get_system_class_loader
            .call(&[])?
            .ok_or("ClassLoader.getSystemClassLoader() returned null".to_string())?;
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
        crate::trace!("Getting method id for {}{}", method_name, signature);

        let method_name_str = CString::new(method_name)?;
        let signature_str = CString::new(signature)?;
        let method = unsafe {
            self.methods.GetMethodID.unwrap()(
                self.env,
                class.class(),
                method_name_str.as_ptr(),
                signature_str.as_ptr(),
            )
        };

        if self.is_err() || method.is_null() {
            Err(self.get_last_error(
                file!(),
                line!(),
                resolve_errors,
                format!("Could not find method '{}{}'", method_name, signature).as_str(),
            )?)
        } else {
            Ok(JavaMethod::new(
                method,
                class,
                JavaType::from_method_return_type(signature)?,
                false,
                #[cfg(feature = "type_check")]
                Signature::from_jni(signature)?,
                #[cfg(feature = "type_check")]
                method_name.to_string(),
            ))
        }
    }

    pub fn get_static_method_id(
        &self,
        class: &'a JavaClass<'a>,
        method_name: &str,
        signature: &str,
    ) -> ResultType<JavaMethod<'a>> {
        crate::trace!("Getting static method id for {}{}", method_name, signature);

        let method_name_str = CString::new(method_name)?;
        let signature_str = CString::new(signature)?;
        let method = unsafe {
            self.methods.GetStaticMethodID.unwrap()(
                self.env,
                class.class(),
                method_name_str.as_ptr(),
                signature_str.as_ptr(),
            )
        };

        if self.is_err() || method.is_null() {
            Err(self.get_last_error(
                file!(),
                line!(),
                true,
                format!("Could not find method '{}'", method_name).as_str(),
            )?)
        } else {
            Ok(JavaMethod::new(
                method,
                class,
                JavaType::from_method_return_type(signature)?,
                true,
                #[cfg(feature = "type_check")]
                Signature::from_jni(signature)?,
                #[cfg(feature = "type_check")]
                method_name.to_string(),
            ))
        }
    }

    unsafe fn convert_args(
        &self,
        args: JavaArgs,
        #[cfg(feature = "type_check")] signature: &Signature,
    ) -> ResultType<Vec<sys::jvalue>> {
        #[cfg(feature = "type_check")]
        if !signature.matches(&args) {
            Err(format!(
                "The arguments do not match the method signature: ({}) != {}",
                args.into_iter()
                    .map(|a| a.get_type().to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                signature
            )
            .into())
        } else {
            Ok(args.iter().map(|arg| arg.to_java_value().value()).collect())
        }

        #[cfg(not(feature = "type_check"))]
        Ok(args.iter().map(|arg| arg.to_java_value().value()).collect())
    }

    pub fn call_object_method(
        &'a self,
        object: JavaObject<'_>,
        method: &JavaMethod<'_>,
        args: JavaArgs,
    ) -> ResultType<Option<LocalJavaObject<'a>>> {
        self.call_object_method_with_errors(object, method, args, true)
    }

    fn return_method_object(
        &'a self,
        obj: sys::jobject,
        _method: &JavaMethod<'_>,
    ) -> ResultType<Option<LocalJavaObject<'a>>> {
        Ok(obj.is_null().not().then(|| {
            LocalJavaObject::new(
                obj,
                self,
                #[cfg(feature = "type_check")]
                _method.get_signature().get_return_type().clone(),
            )
        }))
    }

    pub fn call_object_method_with_errors(
        &'a self,
        object: JavaObject<'_>,
        method: &JavaMethod<'_>,
        args: JavaArgs<'_>,
        resolve_errors: bool,
    ) -> ResultType<Option<LocalJavaObject<'a>>> {
        #[cfg(feature = "log")]
        crate::trace!(
            "Calling object method {} with {} args",
            method.get_java_signature(),
            args.len()
        );

        let obj = unsafe {
            let args = self.convert_args(
                args,
                #[cfg(feature = "type_check")]
                method.get_signature(),
            )?;

            self.methods.CallObjectMethodA.unwrap()(
                self.env,
                object.get_raw(),
                method.id(),
                args.as_ptr(),
            )
        };

        if self.is_err() {
            Err(self.get_last_error(
                file!(),
                line!(),
                resolve_errors,
                "CallObjectMethodA failed",
            )?)
        } else {
            self.return_method_object(obj, method)
        }
    }

    pub fn call_static_object_method(
        &'a self,
        class: &JavaClass<'_>,
        method: &JavaMethod<'_>,
        args: JavaArgs<'_>,
    ) -> ResultType<Option<LocalJavaObject<'a>>> {
        #[cfg(feature = "log")]
        crate::trace!(
            "Calling static object method {} with {} args",
            method.get_java_signature(),
            args.len()
        );

        let obj = unsafe {
            let args = self.convert_args(
                args,
                #[cfg(feature = "type_check")]
                method.get_signature(),
            )?;

            self.methods.CallStaticObjectMethodA.unwrap()(
                self.env,
                class.class(),
                method.id(),
                args.as_ptr(),
            )
        };

        if self.is_err() {
            Err(self.get_last_error(
                file!(),
                line!(),
                true,
                format!("CallStaticObjectMethod failed").as_str(),
            )?)
        } else {
            self.return_method_object(obj, method)
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
        #[cfg(feature = "log")]
        crate::trace!("Getting field {} with signature {}", name, signature);

        let field_name = CString::new(name.clone())?;
        let field_signature = CString::new(signature.to_jni_type())?;
        let field = unsafe {
            if is_static {
                self.methods.GetStaticFieldID.unwrap()(
                    self.env,
                    class.class(),
                    field_name.as_ptr(),
                    field_signature.as_ptr(),
                )
            } else {
                self.methods.GetFieldID.unwrap()(
                    self.env,
                    class.class(),
                    field_name.as_ptr(),
                    field_signature.as_ptr(),
                )
            }
        };

        if self.is_err() || field.is_null() {
            Err(self.get_last_error(
                file!(),
                line!(),
                true,
                format!("Could not find field '{}'", name).as_str(),
            )?)
        } else {
            Ok(unsafe { JavaField::new(field, signature, class, is_static) })
        }
    }

    fn return_field_object<T: GetSignatureRef>(
        &'a self,
        res: sys::jobject,
        _field: &T,
        alt_text: &'static str,
    ) -> ResultType<Option<JavaObject<'a>>> {
        if self.is_err() {
            Err(self.get_last_error(file!(), line!(), true, alt_text)?)
        } else if res.is_null() {
            Ok(None)
        } else {
            Ok(Some(JavaObject::from(LocalJavaObject::new(
                res,
                self,
                #[cfg(feature = "type_check")]
                _field.get_signature_ref().clone(),
            ))))
        }
    }

    pub fn get_object_field(
        &'a self,
        field: &JavaObjectField<'_>,
        object: &JavaObject<'_>,
    ) -> ResultType<Option<JavaObject<'a>>> {
        #[cfg(feature = "log")]
        crate::trace!("Getting object field {}", field.get_signature_ref());

        let res =
            unsafe { self.methods.GetObjectField.unwrap()(self.env, object.get_raw(), field.id()) };

        self.return_field_object(res, field, "GetObjectField failed")
    }

    pub fn set_object_field(
        &'a self,
        field: &JavaObjectField<'_>,
        object: &JavaObject<'_>,
        value: Option<JavaObject<'_>>,
    ) -> ResultType<()> {
        #[cfg(feature = "log")]
        crate::trace!("Setting object field {}", field.get_signature_ref());

        unsafe {
            self.methods.SetObjectField.unwrap()(
                self.env,
                object.get_raw(),
                field.id(),
                value
                    .as_ref()
                    .map(|v| v.get_raw())
                    .unwrap_or(ptr::null_mut()),
            );
        }

        if self.is_err() {
            Err(self.get_last_error(file!(), line!(), true, "SetObjectField failed")?)
        } else {
            Ok(())
        }
    }

    pub fn get_static_object_field(
        &'a self,
        field: &StaticJavaObjectField,
        class: &JavaClass<'_>,
    ) -> ResultType<Option<JavaObject<'a>>> {
        #[cfg(feature = "log")]
        crate::trace!("Getting static object field {}", field.get_signature());

        let res = unsafe {
            self.methods.GetStaticObjectField.unwrap()(self.env, class.class(), field.id())
        };

        self.return_field_object(res, field, "GetStaticObjectField failed")
    }

    pub fn set_static_object_field(
        &'a self,
        field: &StaticJavaObjectField,
        class: &JavaClass<'_>,
        value: Option<JavaObject<'_>>,
    ) -> ResultType<()> {
        #[cfg(feature = "log")]
        crate::trace!("Setting static object field {}", field.get_signature());

        unsafe {
            self.methods.SetStaticObjectField.unwrap()(
                self.env,
                class.class(),
                field.id(),
                value
                    .as_ref()
                    .map(|v| v.get_raw())
                    .unwrap_or(ptr::null_mut()),
            );
        }

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
    ) -> ResultType<Option<LocalJavaObject<'a>>> {
        let obj = unsafe {
            self.methods.GetObjectArrayElement.unwrap()(self.env, array.get_raw(), index)
        };

        if self.is_err() {
            return Err(self.get_last_error(
                file!(),
                line!(),
                resolve_errors,
                "GetObjectArrayElement failed",
            )?);
        }

        if obj.is_null() {
            Ok(None)
        } else {
            Ok(Some(LocalJavaObject::new(
                obj,
                self,
                #[cfg(feature = "type_check")]
                array
                    .get_signature()
                    .inner()
                    .ok_or("Array signature is not an array".to_string())?
                    .lock()
                    .unwrap()
                    .clone(),
            )))
        }
    }

    pub fn set_object_array_element(
        &'a self,
        array: &'a JavaArray<'a>,
        index: i32,
        element: Option<JavaObject<'a>>,
    ) -> ResultType<()> {
        unsafe {
            self.methods.SetObjectArrayElement.unwrap()(
                self.env,
                array.get_raw(),
                index,
                element
                    .as_ref()
                    .map(|e| e.get_raw())
                    .unwrap_or(ptr::null_mut()),
            );
        }

        if self.is_err() {
            Err(self.get_last_error(file!(), line!(), true, "SetObjectArrayElement failed")?)
        } else {
            Ok(())
        }
    }

    pub fn get_array_length(&self, array: sys::jobject) -> ResultType<i32> {
        assert_non_null!(array);
        let length = unsafe { self.methods.GetArrayLength.unwrap()(self.env, array) };

        if self.is_err() {
            Err(self.get_last_error(file!(), line!(), true, "GetArrayLength failed")?)
        } else {
            Ok(length)
        }
    }

    pub fn create_object_array(
        &self,
        class: &'a JavaClass<'a>,
        len: i32,
    ) -> ResultType<JavaObjectArray> {
        let arr = unsafe {
            self.methods.NewObjectArray.unwrap()(self.env, len, class.class(), ptr::null_mut())
        };

        if self.is_err() || arr.is_null() {
            Err(self.get_last_error(file!(), line!(), true, "NewObjectArray failed")?)
        } else {
            Ok(JavaObjectArray::from(LocalJavaObject::new(
                arr,
                self,
                #[cfg(feature = "type_check")]
                JavaType::array(class.get_signature().clone()),
            )))
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
        ReleaseShortArrayElements,
        "short[]"
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
        ReleaseIntArrayElements,
        "int[]"
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
        ReleaseLongArrayElements,
        "long[]"
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
        ReleaseFloatArrayElements,
        "float[]"
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
        ReleaseDoubleArrayElements,
        "double[]"
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
        ReleaseBooleanArrayElements,
        "boolean[]"
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
        ReleaseByteArrayElements,
        "byte[]"
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
        ReleaseCharArrayElements,
        "char[]"
    );

    pub unsafe fn get_string_utf_chars(&self, string: sys::jstring) -> ResultType<String> {
        assert_non_null!(string);
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

    pub fn string_to_java_string<T: Into<Vec<u8>> + Display>(
        &'a self,
        string: T,
    ) -> ResultType<JavaString<'a>> {
        crate::trace!("Converting '{}' to java string", string);

        let c_string = CString::new(string)?;
        let string = unsafe { self.methods.NewStringUTF.unwrap()(self.env, c_string.as_ptr()) };

        if self.is_err() || string.is_null() {
            self.clear_err();
            Err(JNIError::from("NewStringUTF failed").into())
        } else {
            Ok(JavaString::new(self, string))
        }
    }

    pub(in crate::java) fn get_java_vm(&self) -> ResultType<JavaVM> {
        Ok(JavaVM::from_existing(
            self.jvm
                .as_ref()
                .ok_or("jvm was unset".to_string())?
                .clone(),
        ))
    }

    pub unsafe fn is_assignable_from(
        &self,
        sub: sys::jclass,
        sup: sys::jclass,
    ) -> ResultType<bool> {
        assert_non_null!(sub);
        assert_non_null!(sup);
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
        #[cfg(feature = "log")]
        crate::trace!(
            "Creating new instance of {} with {} args",
            constructor.get_signature(),
            args.len()
        );

        let res = unsafe {
            let args = self.convert_args(
                args,
                #[cfg(feature = "type_check")]
                constructor.get_signature(),
            )?;
            self.methods.NewObjectA.unwrap()(
                self.env,
                constructor.class(),
                constructor.id(),
                args.as_ptr(),
            )
        };

        if self.is_err() || res == ptr::null_mut() {
            Err(self.get_last_error(file!(), line!(), true, "NewObjectA failed")?)
        } else {
            Ok(LocalJavaObject::new(
                res,
                self,
                #[cfg(feature = "type_check")]
                constructor.get_class().get_signature().clone(),
            ))
        }
    }

    pub fn get_constructor(
        &'a self,
        class: &'a JavaClass<'a>,
        signature: &str,
    ) -> ResultType<JavaConstructor<'a>> {
        #[cfg(feature = "log")]
        crate::trace!(
            "Getting constructor {} for class {}",
            signature,
            class.get_signature()
        );

        let id = unsafe {
            self.methods.GetMethodID.unwrap()(
                self.env,
                class.class(),
                CString::new("<init>")?.as_ptr(),
                CString::new(signature)?.as_ptr(),
            )
        };

        if self.is_err() || id == ptr::null_mut() {
            Err(self.get_last_error(file!(), line!(), true, "GetMethodID failed")?)
        } else {
            Ok(JavaConstructor::new(
                id,
                class,
                #[cfg(feature = "type_check")]
                Signature::from_jni(signature)?,
            ))
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
        let mut urls = self.create_object_array(&url_class, paths.len() as i32)?;
        for i in 0..paths.len() {
            let java_path = self.string_to_java_string(paths.get(i).unwrap().clone())?;
            let file = self.new_instance(&file_constructor, &[java_path.as_arg()])?;

            let uri = to_uri
                .call(JavaObject::from(file), &[])?
                .ok_or("File.toURI returned null".to_string())?;
            let url = to_url
                .call(JavaObject::from(uri), &[])?
                .ok_or("URI.toURL returned null".to_string())?;

            urls.set(i as i32, Some(JavaObject::from(url)))?;
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
            &[urls.as_arg(), old_class_loader.as_arg()],
        )?;

        let res = GlobalJavaObject::try_from(class_loader)?;
        self.replace_class_loader(res)
    }

    pub fn replace_class_loader(&self, loader: GlobalJavaObject) -> ResultType<()> {
        self.jvm
            .as_ref()
            .ok_or("The jvm was unset".to_string())?
            .lock()
            .unwrap()
            .set_class_loader(loader);

        Ok(())
    }

    pub fn get_vm_ptr(&self) -> ResultType<Arc<Mutex<JavaVMPtr>>> {
        Ok(self
            .jvm
            .as_ref()
            .ok_or("The jvm was unset".to_string())?
            .clone())
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
    }
}
