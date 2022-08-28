use crate::jni::objects::class::{GlobalJavaClass, JavaClass};
use crate::jni::objects::object::GlobalJavaObject;
use crate::jni::objects::string::JavaString;
use crate::jni::objects::value::JavaInt;
use crate::jni::traits::GetSignature;
use crate::tests::common::get_vm;

#[test]
fn local_class_by_name() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/String", &env).unwrap();

    let value_of = class
        .get_static_object_method("valueOf", "(I)Ljava/lang/String;")
        .unwrap();
    let int = JavaInt::new(1234);

    let str = value_of.call(vec![Box::new(&int)]).unwrap();
    let string = JavaString::from(str);

    assert_eq!(string.to_string().unwrap(), "1234");
}

#[test]
fn local_to_global_class() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/String", &env).unwrap();

    let cls = GlobalJavaClass::try_from(class).unwrap();
    let local = JavaClass::from_global(&cls, &env);

    let value_of = local
        .get_static_object_method("valueOf", "(I)Ljava/lang/String;")
        .unwrap();
    let int = JavaInt::new(1234);

    let str = value_of.call(vec![Box::new(&int)]).unwrap();
    let string = JavaString::from(str);

    assert_eq!(string.to_string().unwrap(), "1234");
}

#[test]
fn global_class_by_name() {
    let env = get_vm().attach_thread().unwrap();
    let cls = GlobalJavaClass::by_name("java.lang.String", &env).unwrap();
    let local = JavaClass::from_global(&cls, &env);

    let value_of = local
        .get_static_object_method("valueOf", "(I)Ljava/lang/String;")
        .unwrap();
    let int = JavaInt::new(1234);

    let str = value_of.call(vec![Box::new(&int)]).unwrap();
    let string = JavaString::from(str);

    assert_eq!(string.to_string().unwrap(), "1234");
}

#[test]
fn create_global_object() {
    let env = get_vm().attach_thread().unwrap();
    let str = JavaString::try_from("test".to_string(), &env).unwrap();
    let global = GlobalJavaObject::try_from(str).unwrap();

    let local = JavaString::from_global(&global, &env);
    assert_eq!(local.to_string().unwrap(), "test");
}

#[test]
fn clone_global_object() {
    let env = get_vm().attach_thread().unwrap();
    let str = JavaString::try_from("test".to_string(), &env).unwrap();
    let global = GlobalJavaObject::try_from(str).unwrap();
    let clone = global.clone();

    let local = JavaString::from_global(&clone, &env);
    assert_eq!(local.to_string().unwrap(), "test");
}

#[test]
fn get_signature() {
    let env = get_vm().attach_thread().unwrap();
    let str = JavaString::try_from("test".to_string(), &env).unwrap();

    let signature = str.to_object().get_signature().unwrap();
    assert_eq!(signature.to_string(), "java.lang.String");
}
