use crate::java::objects::array::{JavaObjectArray, JavaShortArray};
use crate::java::objects::class::JavaClass;
use crate::java::objects::java_object::JavaObject;
use crate::java::objects::string::JavaString;
use crate::objects::java_object::AsJavaObject;
use crate::tests::common::get_vm;

fn check_string_array(array: &JavaObjectArray) {
    for i in 0..array.len().unwrap() {
        let str = array.get(i).unwrap().unwrap();
        let string = JavaString::try_from(str).unwrap();
        assert_eq!(string.to_string().unwrap(), format!("test{}", i + 1));
    }
}

#[test]
fn object_array_from_objects() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/String", &env).unwrap();

    let str1 = JavaString::from_string("test1".to_string(), &env).unwrap();
    let str2 = JavaString::from_string("test2".to_string(), &env).unwrap();
    let str3 = JavaString::from_string("test3".to_string(), &env).unwrap();

    let data = vec![
        Some(str1.as_java_object()),
        Some(str2.as_java_object()),
        Some(str3.as_java_object()),
    ];
    let array = JavaObjectArray::from_vec(data, &class).unwrap();

    assert_eq!(array.len().unwrap(), 3);

    check_string_array(&array);
}

#[test]
fn empty_object_array() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/String", &env).unwrap();

    let array = JavaObjectArray::new(&class, 5).unwrap();

    assert_eq!(array.len().unwrap(), 5);

    for i in 0..array.len().unwrap() {
        assert!(array.get(i).unwrap().is_none());
    }
}

#[test]
fn object_array_set() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/String", &env).unwrap();

    let str1 = JavaString::from_string("test1".to_string(), &env).unwrap();
    let str2 = JavaString::from_string("test2".to_string(), &env).unwrap();
    let str3 = JavaString::from_string("test3".to_string(), &env).unwrap();
    let str4 = JavaString::from_string("test4".to_string(), &env).unwrap();
    let str5 = JavaString::from_string("test5".to_string(), &env).unwrap();

    let mut array = JavaObjectArray::new(&class, 5).unwrap();

    array.set(0, Some(JavaObject::from(&str1))).unwrap();
    array.set(1, Some(JavaObject::from(&str2))).unwrap();
    array.set(2, Some(JavaObject::from(&str3))).unwrap();
    array.set(3, Some(JavaObject::from(&str4))).unwrap();
    array.set(4, Some(JavaObject::from(&str5))).unwrap();

    assert_eq!(array.len().unwrap(), 5);
    check_string_array(&array);
}

#[test]
fn short_array() {
    let env = get_vm().attach_thread().unwrap();
    let arr = JavaShortArray::new(&env, &[1, 2, 3, 4, 5]).unwrap();

    assert_eq!(arr.len().unwrap(), 5);

    let data = arr.get_data().unwrap();
    assert_eq!(data.len(), 5);

    for (i, item) in data.iter().enumerate() {
        assert_eq!(*item, i as i16 + 1);
    }
}
