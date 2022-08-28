use crate::jni::objects::class::JavaClass;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::string::JavaString;
use crate::jni::objects::value::{JavaBoolean, JavaInt};
use crate::tests::common::get_vm;

#[test]
fn create_env() {
    get_vm().attach_thread().unwrap();
}

#[test]
fn java_string() {
    let env = get_vm().attach_thread().unwrap();
    let str = JavaString::try_from("Test".to_string(), &env).unwrap();

    assert_eq!(str.to_string().unwrap(), "Test");
}

#[test]
fn string_value_of() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/String", &env).unwrap();

    let value_of = class
        .get_static_object_method("valueOf", "(Z)Ljava/lang/String;")
        .unwrap();
    let bool = JavaBoolean::new(true);
    let string = value_of.call(vec![Box::new(&bool)]).unwrap();

    let str = JavaString::from(string);
    assert_eq!(str.to_string().unwrap(), "true");
}

#[test]
fn string_index_of() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/String", &env).unwrap();

    let string = JavaString::try_from("test".to_string(), &env).unwrap();
    let index_of = class
        .get_int_method("indexOf", "(I)I")
        .unwrap()
        .bind(JavaObject::from(&string));

    let char = JavaInt::new('s' as i32);
    let index = index_of.call(vec![Box::new(&char)]).unwrap();

    assert_eq!(index, 2);
}
