use java_rs::java_vm::JavaVM;
use lazy_static::lazy_static;

lazy_static! {
    static ref VM: JavaVM = JavaVM::new("1.8", None, &[]).unwrap();
}

pub fn get_vm() -> JavaVM {
    VM.clone()
}
