use crate::java::objects::class::{GlobalJavaClass, JavaClass};
use crate::java::objects::java_object::JavaObject;
use crate::java::objects::method::{GlobalJavaMethod, JavaCharMethod, JavaIntMethod};
use crate::java::objects::string::JavaString;
use crate::java::objects::value::{JavaByte, JavaChar, JavaInt};
use crate::objects::args::AsJavaArg;
use crate::tests::common::get_vm;

macro_rules! get_integer {
    ($class_name: ident, $result_name: ident) => {
        let env = get_vm().attach_thread().unwrap();
        let $class_name = JavaClass::by_name("java/lang/Integer", &env).unwrap();
        let value_of = $class_name
            .get_static_object_method("valueOf", "(I)Ljava/lang/Integer;")
            .unwrap();
        let int = JavaInt::new(1234);

        let $result_name = value_of.call(&[Box::new(&int)]).unwrap().unwrap();
    };
}

#[test]
fn int_method() {
    get_integer!(cls, int);

    let int_value = cls
        .get_int_method("intValue", "()I")
        .unwrap()
        .bind(JavaObject::from(&int));
    assert_eq!(int_value.call(&[]).unwrap(), 1234);
}

#[test]
fn long_method() {
    get_integer!(cls, int);

    let long_value = cls
        .get_long_method("longValue", "()J")
        .unwrap()
        .bind(JavaObject::from(&int));
    assert_eq!(long_value.call(&[]).unwrap(), 1234 as i64);
}

#[test]
fn double_method() {
    get_integer!(cls, int);

    let double_value = cls
        .get_double_method("doubleValue", "()D")
        .unwrap()
        .bind(JavaObject::from(&int));
    assert_eq!(double_value.call(&[]).unwrap(), 1234 as f64);
}

#[test]
fn float_method() {
    get_integer!(cls, int);

    let float_value = cls
        .get_float_method("floatValue", "()F")
        .unwrap()
        .bind(JavaObject::from(&int));
    assert_eq!(float_value.call(&[]).unwrap(), 1234 as f32);
}

#[test]
fn boolean_method() {
    get_integer!(cls, int);

    let boolean_value = cls
        .get_boolean_method("equals", "(Ljava/lang/Object;)Z")
        .unwrap()
        .bind(JavaObject::from(&int));
    assert_eq!(boolean_value.call(&[int.as_arg()]).unwrap(), true);
}

#[test]
fn short_method() {
    get_integer!(cls, int);

    let short_value = cls
        .get_short_method("shortValue", "()S")
        .unwrap()
        .bind(JavaObject::from(&int));
    assert_eq!(short_value.call(&[]).unwrap(), 1234 as i16);
}

#[test]
fn byte_method() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/Byte", &env).unwrap();
    let value_of = class
        .get_static_object_method("valueOf", "(B)Ljava/lang/Byte;")
        .unwrap();

    let byte = JavaByte::new(123);
    let result = value_of.call(&[Box::new(&byte)]).unwrap().unwrap();
    let byte_value = class
        .get_byte_method("byteValue", "()B")
        .unwrap()
        .bind(JavaObject::from(&result));

    assert_eq!(byte_value.call(&[]).unwrap(), 123);
}

#[test]
fn char_method() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/Character", &env).unwrap();
    let value_of = class
        .get_static_object_method("valueOf", "(C)Ljava/lang/Character;")
        .unwrap();

    let char = JavaChar::new('a' as u16);
    let result = value_of.call(&[Box::new(&char)]).unwrap().unwrap();
    let char_value = class
        .get_char_method("charValue", "()C")
        .unwrap()
        .bind(JavaObject::from(&result));

    assert_eq!(char_value.call(&[]).unwrap(), 'a' as u16);
}

macro_rules! get_parse_method {
    ($class_name: expr, $method_name: expr, $signature: expr, $method: ident, $result_var: ident) => {
        let env = get_vm().attach_thread().unwrap();
        let cls = JavaClass::by_name($class_name, &env).unwrap();
        let str = JavaString::from_string("123".to_string(), &env).unwrap();
        let method = cls.$method($method_name, $signature).unwrap();

        let $result_var = method.call(&[Box::new(&str)]).unwrap();
    };
}

#[test]
fn static_int_method() {
    get_parse_method!(
        "java/lang/Integer",
        "parseInt",
        "(Ljava/lang/String;)I",
        get_static_int_method,
        int
    );
    assert_eq!(int, 123);
}

#[test]
fn static_long_method() {
    get_parse_method!(
        "java/lang/Long",
        "parseLong",
        "(Ljava/lang/String;)J",
        get_static_long_method,
        long
    );
    assert_eq!(long, 123);
}

#[test]
fn static_double_method() {
    get_parse_method!(
        "java/lang/Double",
        "parseDouble",
        "(Ljava/lang/String;)D",
        get_static_double_method,
        double
    );
    assert_eq!(double, 123.0);
}

#[test]
fn static_float_method() {
    get_parse_method!(
        "java/lang/Float",
        "parseFloat",
        "(Ljava/lang/String;)F",
        get_static_float_method,
        float
    );
    assert_eq!(float, 123.0);
}

#[test]
fn static_short_method() {
    get_parse_method!(
        "java/lang/Short",
        "parseShort",
        "(Ljava/lang/String;)S",
        get_static_short_method,
        short
    );
    assert_eq!(short, 123);
}

#[test]
fn static_boolean_method() {
    let env = get_vm().attach_thread().unwrap();
    let cls = JavaClass::by_name("java/lang/Boolean", &env).unwrap();
    let str = JavaString::from_string("true".to_string(), &env).unwrap();
    let method = cls
        .get_static_boolean_method("parseBoolean", "(Ljava/lang/String;)Z")
        .unwrap();

    let boolean = method.call(&[Box::new(&str)]).unwrap();
    assert_eq!(boolean, true);
}

#[test]
fn static_byte_method() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/Byte", &env).unwrap();
    let parse_byte = class
        .get_static_byte_method("parseByte", "(Ljava/lang/String;)B")
        .unwrap();

    let str = JavaString::from_string("123".to_string(), &env).unwrap();
    let byte = parse_byte.call(&[Box::new(&str)]).unwrap();
    assert_eq!(byte, 123);
}

#[test]
fn static_char_method() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/Character", &env).unwrap();
    let parse_char = class.get_static_char_method("toLowerCase", "(C)C").unwrap();

    let c = JavaChar::new('A' as u16);
    let char = parse_char.call(&[Box::new(&c)]).unwrap();
    assert_eq!(char, 'a' as u16);
}

#[test]
fn global_method_assign_int() {
    let env = get_vm().attach_thread().unwrap();
    let class = JavaClass::by_name("java/lang/Integer", &env).unwrap();

    let int_value = class.get_int_method("intValue", "()I").unwrap();
    let global_class = GlobalJavaClass::by_name("java/lang/Integer", &env).unwrap();
    let global = GlobalJavaMethod::from(global_class, int_value.into());

    assert!(JavaIntMethod::from_global(global.clone(), &class).is_ok());
    assert!(JavaCharMethod::from_global(global, &class).is_err());
}
