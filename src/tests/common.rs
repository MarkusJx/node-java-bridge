use crate::jni::java_vm::{InternalJavaOptions, JavaVM};
use crate::node::java_options::JavaOptions;
use lazy_static::lazy_static;

lazy_static! {
    static ref VM: JavaVM = JavaVM::new(
        &"1.8".to_string(),
        None,
        &vec![],
        InternalJavaOptions::from(JavaOptions::default())
    )
    .unwrap();
}

pub fn get_vm() -> JavaVM {
    VM.clone()
}
