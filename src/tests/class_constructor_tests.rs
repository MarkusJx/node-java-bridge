use crate::java::class_constructor::ClassConstructor;
use crate::tests::common::get_vm;
use java_rs::java_type::Type;
use java_rs::objects::args::AsJavaArg;
use java_rs::objects::object::LocalJavaObject;
use java_rs::objects::string::JavaString;

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

    let str = JavaString::from_string("test".to_string(), &env).unwrap();
    let instance = default.new_instance(&[str.as_arg()]).unwrap();

    let local = JavaString::try_from(LocalJavaObject::from(&instance, &env)).unwrap();

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
