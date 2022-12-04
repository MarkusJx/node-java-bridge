use crate::jni::java_env::JavaEnv;
use crate::jni::objects::args::JavaArgs;
use crate::jni::objects::class::{GlobalJavaClass, JavaClass};
use crate::jni::objects::object::LocalJavaObject;
use crate::jni::util::util::ResultType;
use crate::sys;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct JavaConstructor<'a> {
    method: sys::jmethodID,
    class: &'a JavaClass<'a>,
}

impl<'a> JavaConstructor<'a> {
    pub(in crate::jni) fn new(method: sys::jmethodID, class: &'a JavaClass<'a>) -> Self {
        Self { method, class }
    }

    pub fn new_instance<'b>(
        &self,
        env: &'b JavaEnv<'b>,
        args: JavaArgs,
    ) -> ResultType<LocalJavaObject<'b>> {
        env.get_env().new_instance(self, args)
    }

    pub(in crate::jni) unsafe fn class(&self) -> sys::jclass {
        self.class.class()
    }

    pub(in crate::jni) unsafe fn id(&self) -> sys::jmethodID {
        self.method
    }

    pub fn from_global(global: &GlobalJavaConstructor, class: &'a JavaClass<'a>) -> Self {
        Self {
            method: global.method.load(Ordering::Relaxed),
            class,
        }
    }
}

pub struct GlobalJavaConstructor {
    method: AtomicPtr<sys::_jmethodID>,
    class: GlobalJavaClass,
}

impl GlobalJavaConstructor {
    pub fn from_local(local: JavaConstructor<'_>, class: GlobalJavaClass) -> Self {
        Self {
            method: AtomicPtr::new(local.method),
            class,
        }
    }

    pub fn get_class<'a, 'b>(&'a self, env: &'b JavaEnv<'b>) -> JavaClass<'b>
    where
        'a: 'b,
    {
        JavaClass::from_global(&self.class, env)
    }
}

impl Clone for GlobalJavaConstructor {
    fn clone(&self) -> Self {
        Self {
            method: AtomicPtr::new(self.method.load(Ordering::Relaxed)),
            class: self.class.clone(),
        }
    }
}

unsafe impl Send for GlobalJavaConstructor {}
