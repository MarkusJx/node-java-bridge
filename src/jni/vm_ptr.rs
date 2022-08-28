use crate::jni::java_vm::InternalJavaOptions;
use crate::jni::objects::object::GlobalJavaObject;
use crate::sys;
use std::collections::HashMap;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::thread::ThreadId;

/// The pointer to the java vm.
/// This should only exist once per process.
pub struct JavaVMPtr {
    /// The pointer to the java vm object
    vm: AtomicPtr<sys::JavaVM>,
    /// Additional options for the java vm.
    options: InternalJavaOptions,
    /// Contains the thread ids and the number of
    /// jvm references those threads are holding.
    /// This is used to determine if a thread should be detached.
    references: HashMap<ThreadId, i32>,
    /// The class loader to use for all classes.
    /// This is at first unset but will be set
    /// once the vm creation is done.
    class_loader: Option<GlobalJavaObject>,
    destroy: bool,
}

impl JavaVMPtr {
    pub(in crate::jni) fn new(vm: *mut sys::JavaVM, options: InternalJavaOptions) -> Self {
        let mut res = Self {
            vm: AtomicPtr::new(vm),
            options,
            references: HashMap::new(),
            class_loader: None,
            destroy: true,
        };

        res.increase_ref_count();
        return res;
    }

    pub(in crate::jni) fn from_raw(vm: *mut sys::JavaVM, options: InternalJavaOptions) -> Self {
        Self {
            vm: AtomicPtr::new(vm),
            options,
            references: HashMap::new(),
            class_loader: None,
            destroy: false,
        }
    }

    /// Set the class loader
    pub(in crate::jni) fn set_class_loader(&mut self, class_loader: GlobalJavaObject) {
        self.class_loader = Some(class_loader);
    }

    /// Get the class loader
    pub(in crate::jni) fn class_loader(&self) -> &Option<GlobalJavaObject> {
        &self.class_loader
    }

    /// Get the JNI methods
    pub(in crate::jni) unsafe fn methods(&self) -> sys::JNIInvokeInterface_ {
        *(*self.vm())
    }

    /// Get the JavaVM pointer
    pub(in crate::jni) unsafe fn vm(&self) -> *mut sys::JavaVM {
        self.vm.load(Ordering::Relaxed)
    }

    fn update_ref_count(&mut self, func: fn(count: i32) -> i32) {
        if self.options.use_daemon_threads {
            return;
        }

        let thread_id = std::thread::current().id();
        let count = *self.references.get(&thread_id).unwrap_or(&0);

        self.references
            .insert(std::thread::current().id(), func(count));

        if *self.references.get(&thread_id).unwrap() <= 0 {
            self.references.remove(&thread_id);
            unsafe {
                self.methods().DetachCurrentThread.unwrap()(self.vm());
            }
        }
    }

    /// Increments the reference count for the current thread.
    pub(in crate::jni) fn increase_ref_count(&mut self) {
        self.update_ref_count(|count| count + 1);
    }

    /// Decrements the reference count for the current thread.
    /// If the reference count is 0, the thread will be detached.
    pub(in crate::jni) fn decrease_ref_count(&mut self) {
        self.update_ref_count(|count| count - 1);
    }
}

impl Drop for JavaVMPtr {
    fn drop(&mut self) {
        if self.destroy {
            self.decrease_ref_count();
            unsafe {
                self.methods().DestroyJavaVM.unwrap()(self.vm());
            }
        }
    }
}

unsafe impl Sync for JavaVMPtr {}
