use crate::jni::class_constructor::ClassConstructor;
use crate::jni::java_type::Type;
use crate::jni::objects::object::LocalJavaObject;
use crate::jni::objects::string::JavaString;
use crate::tests::common::get_vm;

#[test]
fn get_constructors() {
    let constructors =
        ClassConstructor::get_constructors(get_vm(), "java.lang.String".to_string()).unwrap();

    assert_eq!(constructors.len(), 15);
}

#[test]
fn new_instance() {
    let env = get_vm().attach_thread().unwrap();
    let constructors =
        ClassConstructor::get_constructors(get_vm(), "java.lang.String".to_string()).unwrap();

    let default = constructors
        .iter()
        .find(|e| {
            e.parameter_types().len() == 1 && e.parameter_types()[0].type_enum() == Type::String
        })
        .unwrap();

    let str = JavaString::try_from("test".to_string(), &env).unwrap();
    let instance = default.new_instance(vec![Box::new(&str)]).unwrap();

    let local = JavaString::from(LocalJavaObject::from(&instance, &env));

    assert_eq!(local.to_string().unwrap(), "test");
}

#[test]
fn to_string() {
    let constructors =
        ClassConstructor::get_constructors(get_vm(), "java.lang.String".to_string()).unwrap();

    assert_eq!(constructors.len(), 15);

    let default = constructors
        .iter()
        .find(|e| {
            e.parameter_types().len() == 1 && e.parameter_types()[0].type_enum() == Type::String
        })
        .unwrap();

    assert_eq!(default.to_string(), "java.lang.String(java.lang.String)");
}
