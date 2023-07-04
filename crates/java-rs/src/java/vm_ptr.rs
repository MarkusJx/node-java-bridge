use crate::java::objects::object::GlobalJavaObject;
use crate::{assert_non_null, sys};
use std::sync::atomic::{AtomicPtr, Ordering};

/// The pointer to the java vm.
/// This should only exist once per process.
pub struct JavaVMPtr {
    /// The pointer to the java vm object
    vm: AtomicPtr<sys::JavaVM>,
    /// The class loader to use for all classes.
    /// This is at first unset but will be set
    /// once the vm creation is done.
    class_loader: Option<GlobalJavaObject>,
    destroy: bool,
}

impl JavaVMPtr {
    pub(in crate::java) fn new(vm: *mut sys::JavaVM) -> Self {
        assert_non_null!(vm);
        Self {
            vm: AtomicPtr::new(vm),
            class_loader: None,
            destroy: true,
        }
    }

    pub(in crate::java) fn from_raw(vm: *mut sys::JavaVM) -> Self {
        assert_non_null!(vm);
        Self {
            vm: AtomicPtr::new(vm),
            class_loader: None,
            destroy: false,
        }
    }

    /// Set the class loader
    pub(in crate::java) fn set_class_loader(&mut self, class_loader: GlobalJavaObject) {
        self.class_loader = Some(class_loader);
    }

    /// Get the class loader
    pub(in crate::java) fn class_loader(&self) -> &Option<GlobalJavaObject> {
        &self.class_loader
    }

    /// Get the JNI methods
    pub(in crate::java) unsafe fn methods(&self) -> sys::JNIInvokeInterface_ {
        *(*self.vm())
    }

    /// Get the JavaVM pointer
    pub(in crate::java) unsafe fn vm(&self) -> *mut sys::JavaVM {
        self.vm.load(Ordering::Relaxed)
    }
}

impl Drop for JavaVMPtr {
    fn drop(&mut self) {
        if self.destroy {
            unsafe {
                self.methods().DestroyJavaVM.unwrap()(self.vm());
            }
        }
    }
}

unsafe impl Sync for JavaVMPtr {}
