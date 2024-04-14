use crate::java::java_env::JavaEnv;
use crate::java::jni_error::JNIError;
use crate::java::objects::class::JavaClass;
use crate::java::objects::object::LocalJavaObject;
use crate::java::objects::value::JavaInt;
use crate::objects::args::AsJavaArg;
use crate::sys;
use std::error::Error;

pub fn jni_error_to_string(error: i32) -> String {
    match error {
        sys::JNI_EDETACHED => "Thread detached from the vm".to_string(),
        sys::JNI_EVERSION => "JNI version error".to_string(),
        sys::JNI_ENOMEM => "Not enough memory".to_string(),
        sys::JNI_EEXIST => "VM already created".to_string(),
        sys::JNI_EINVAL => "Invalid arguments".to_string(),
        _ => "Unknown error".to_string(),
    }
}

pub fn parse_jni_version(version: &str) -> Result<u32, Box<dyn Error + Send + Sync>> {
    match version {
        "1.1" => Ok(65537),
        "1.2" => Ok(65538),
        "1.4" => Ok(65540),
        "1.6" => Ok(65542),
        "1.8" => Ok(65544),
        "9" => Ok(589824),
        "10" => Ok(655360),
        _ => Err(JNIError::from(format!("Unknown JNI version: {}", version)).into()),
    }
}

pub fn jni_version_to_string(version: i32) -> Result<String, Box<dyn Error + Send + Sync>> {
    match version {
        65537 => Ok("1.1".to_string()),
        65538 => Ok("1.2".to_string()),
        65540 => Ok("1.4".to_string()),
        65542 => Ok("1.6".to_string()),
        65544 => Ok("1.8".to_string()),
        589824 => Ok("9".to_string()),
        655360 => Ok("10".to_string()),
        _ => Err(JNIError::from(format!("Unknown JNI version: {}", version)).into()),
    }
}

pub type ResultType<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn jni_type_to_java_type(to_convert: &String) -> String {
    return if to_convert == "Z" || to_convert == "boolean" {
        "boolean".to_string()
    } else if to_convert == "B" || to_convert == "byte" {
        "byte".to_string()
    } else if to_convert == "C" || to_convert == "char" {
        "char".to_string()
    } else if to_convert == "S" || to_convert == "short" {
        "short".to_string()
    } else if to_convert == "I" || to_convert == "int" {
        "int".to_string()
    } else if to_convert == "J" || to_convert == "long" {
        "long".to_string()
    } else if to_convert == "F" || to_convert == "float" {
        "float".to_string()
    } else if to_convert == "D" || to_convert == "double" {
        "double".to_string()
    } else if to_convert == "V" {
        "void".to_string()
    } else if !to_convert.is_empty() && to_convert.chars().nth(0).unwrap() == '[' {
        jni_type_to_java_type(&to_convert.clone()[1..].to_string()) + "[]"
    } else if !to_convert.is_empty() && to_convert.chars().nth(0).unwrap() == 'L' {
        to_convert.clone()[1..(to_convert.len() - 1)].replace('/', ".")
    } else {
        to_convert.clone().replace('/', ".")
    };
}

pub fn method_is_public(
    env: &JavaEnv,
    method: &LocalJavaObject,
    is_method: bool,
    only_static: bool,
) -> ResultType<bool> {
    let class = if is_method {
        JavaClass::by_name("java/lang/reflect/Method", env)?
    } else {
        JavaClass::by_name("java/lang/reflect/Field", env)?
    };

    let get_modifiers = class
        .get_int_method("getModifiers", "()I")?
        .bind(method.into());
    let modifier = JavaClass::by_name("java/lang/reflect/Modifier", env)?;
    let is_public = modifier.get_static_boolean_method("isPublic", "(I)Z")?;
    let is_static = modifier.get_static_boolean_method("isStatic", "(I)Z")?;

    let modifiers = get_modifiers.call(&[])?;
    let is_public = is_public.call(&[JavaInt::new(modifiers).as_arg()])?;
    let is_static = is_static.call(&[JavaInt::new(modifiers).as_arg()])?;

    Ok(is_public && is_static == only_static)
}

pub fn field_is_final(env: &JavaEnv, field: &LocalJavaObject) -> ResultType<bool> {
    let method_class = JavaClass::by_name("java/lang/reflect/Field", env)?;
    let get_modifiers = method_class
        .get_int_method("getModifiers", "()I")?
        .bind(field.into());
    let modifier = JavaClass::by_name("java/lang/reflect/Modifier", env)?;

    let is_final = modifier.get_static_boolean_method("isFinal", "(I)Z")?;
    let modifiers = get_modifiers.call(&[])?;
    is_final.call(&[JavaInt::new(modifiers).as_arg()])
}
