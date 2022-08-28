use crate::jni::objects::array::{JavaObjectArray, JavaShortArray};
use crate::jni::objects::class::JavaClass;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::string::JavaString;
use crate::jni::traits::IsNull;
use crate::tests::common::get_vm;

fn check_string_array(array: &JavaObjectArray) {
    for i in 0..array.len().unwrap() {
        let str = array.get(i).unwrap();
        let string = JavaString::from(str);
        assert_eq!(string.to_string().unwrap(), format!("test{}", i + 1));
    }
}

#[test]
fn object_array_from_objects() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/String", &env).unwrap();

    let str1 = JavaString::try_from("test1".to_string(), &env).unwrap();
    let str2 = JavaString::try_from("test2".to_string(), &env).unwrap();
    let str3 = JavaString::try_from("test3".to_string(), &env).unwrap();

    let data = vec![
        JavaObject::from(&str1),
        JavaObject::from(&str2),
        JavaObject::from(&str3),
    ];
    let array = JavaObjectArray::from_vec(data, &class).unwrap();

    assert!(!array.is_null());
    assert_eq!(array.len().unwrap(), 3);

    check_string_array(&array);
}

#[test]
fn empty_object_array() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/String", &env).unwrap();

    let array = JavaObjectArray::new(&class, 5).unwrap();

    assert!(!array.is_null());
    assert_eq!(array.len().unwrap(), 5);

    for i in 0..array.len().unwrap() {
        assert!(array.get(i).unwrap().is_null());
    }
}

#[test]
fn object_array_set() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/String", &env).unwrap();

    let str1 = JavaString::try_from("test1".to_string(), &env).unwrap();
    let str2 = JavaString::try_from("test2".to_string(), &env).unwrap();
    let str3 = JavaString::try_from("test3".to_string(), &env).unwrap();
    let str4 = JavaString::try_from("test4".to_string(), &env).unwrap();
    let str5 = JavaString::try_from("test5".to_string(), &env).unwrap();

    let mut array = JavaObjectArray::new(&class, 5).unwrap();

    array.set(0, JavaObject::from(&str1)).unwrap();
    array.set(1, JavaObject::from(&str2)).unwrap();
    array.set(2, JavaObject::from(&str3)).unwrap();
    array.set(3, JavaObject::from(&str4)).unwrap();
    array.set(4, JavaObject::from(&str5)).unwrap();

    assert_eq!(array.len().unwrap(), 5);
    check_string_array(&array);
}

#[test]
fn short_array() {
    let env = get_vm().attach_thread().unwrap();
    let arr = JavaShortArray::new(&env, &vec![1, 2, 3, 4, 5]).unwrap();

    assert!(!arr.is_null());
    assert_eq!(arr.len().unwrap(), 5);

    let data = arr.get_data().unwrap();
    assert_eq!(data.len(), 5);

    for i in 0..data.len() {
        assert_eq!(data[i], i as i16 + 1);
    }
}
