use crate::objects::args::AsJavaArg;
use crate::objects::string::JavaString;
use crate::objects::value::JavaInt;
#[cfg(feature = "type_check")]
use crate::signature::Signature;
use crate::tests::common::get_vm;

#[test]
fn parse_signature() {
    let sig = Signature::from_jni("(Ljava/lang/String;I)Ljava/lang/String;").unwrap();
    assert_eq!(sig.num_args(), 2);
    assert_eq!(sig.get_args()[0].to_string(), "java.lang.String");
    assert_eq!(sig.get_args()[1].to_string(), "int");
    assert_eq!(sig.get_return_type().to_string(), "java.lang.String");
}

#[test]
fn parse_signature_with_array() {
    let sig = Signature::from_jni("([Ljava/lang/String;I)Ljava/lang/String;").unwrap();
    assert_eq!(sig.num_args(), 2);
    assert_eq!(sig.get_args()[0].to_string(), "java.lang.String[]");
    assert!(sig.get_args()[0].is_array());
    assert_eq!(sig.get_args()[1].to_string(), "int");
    assert_eq!(sig.get_return_type().to_string(), "java.lang.String");
}

#[test]
fn parse_signature_with_void() {
    let sig = Signature::from_jni("()V").unwrap();
    assert_eq!(sig.num_args(), 0);
    assert_eq!(sig.get_return_type().to_string(), "void");
}

#[test]
fn parse_signature_with_void_array() {
    let sig = Signature::from_jni("()[V").unwrap();
    assert_eq!(sig.num_args(), 0);
    assert_eq!(sig.get_return_type().to_string(), "void[]");
    assert!(sig.get_return_type().is_array());
}

#[test]
fn parse_signature_with_void_array_array() {
    let sig = Signature::from_jni("()[[V").unwrap();
    assert_eq!(sig.num_args(), 0);
    assert_eq!(sig.get_return_type().to_string(), "void[][]");
    assert!(sig.get_return_type().is_array());
}

#[test]
fn parse_signature_with_int_array() {
    let sig = Signature::from_jni("()[I").unwrap();
    assert_eq!(sig.num_args(), 0);
    assert_eq!(sig.get_return_type().to_string(), "int[]");
    assert!(sig.get_return_type().is_array());
}

#[test]
fn parse_signature_with_int_array_array() {
    let sig = Signature::from_jni("()[[I").unwrap();
    assert_eq!(sig.num_args(), 0);
    assert_eq!(sig.get_return_type().to_string(), "int[][]");
    assert!(sig.get_return_type().is_array());
}

#[test]
fn parse_signature_with_int_array_array_array() {
    let sig = Signature::from_jni("()[[[I").unwrap();
    assert_eq!(sig.num_args(), 0);
    assert_eq!(sig.get_return_type().to_string(), "int[][][]");
    assert!(sig.get_return_type().is_array());
}

#[test]
fn parse_signature_with_boolean_array() {
    let sig = Signature::from_jni("()[Z").unwrap();
    assert_eq!(sig.num_args(), 0);
    assert_eq!(sig.get_return_type().to_string(), "boolean[]");
    assert!(sig.get_return_type().is_array());
}

#[test]
fn parse_signature_with_many_args() {
    let sig = Signature::from_jni("(Ljava/lang/String;I[[I[[[Z)V").unwrap();
    assert_eq!(sig.num_args(), 4);
    assert_eq!(sig.get_args()[0].to_string(), "java.lang.String");
    assert_eq!(sig.get_args()[1].to_string(), "int");
    assert_eq!(sig.get_args()[2].to_string(), "int[][]");
    assert_eq!(sig.get_args()[3].to_string(), "boolean[][][]");
    assert_eq!(sig.get_return_type().to_string(), "void");
}

#[test]
fn method_with_invalid_number_of_args() {
    let env = get_vm().attach_thread().unwrap();
    let cls = env.find_class("java/lang/String").unwrap();
    let method = cls
        .get_static_object_method("valueOf", "(I)Ljava/lang/String;")
        .unwrap();

    method
        .call(&[JavaInt::from(0).as_arg(), JavaInt::new(0).as_arg()])
        .expect_err("Should have failed");
}

#[test]
fn method_call_with_invalid_args() {
    let env = get_vm().attach_thread().unwrap();
    let cls = env.find_class("java/lang/String").unwrap();
    let method = cls
        .get_static_object_method("valueOf", "(I)Ljava/lang/String;")
        .unwrap();
    method
        .call(&[JavaString::from_string("hello".into(), &env)
            .unwrap()
            .as_arg()])
        .expect_err("Should have failed");
}
