use crate::java::java_env::JavaEnv;
use crate::java::java_field::GlobalJavaField;
use crate::java::java_type::JavaType;
use crate::java::objects::array::JavaObjectArray;
use crate::java::objects::class::{GlobalJavaClass, JavaClass};
use crate::java::objects::constructor::GlobalJavaConstructor;
use crate::java::objects::java_object::JavaObject;
use crate::java::objects::method::GlobalJavaMethod;
use crate::java::objects::object::LocalJavaObject;
use crate::java::objects::string::JavaString;
use crate::java::util::helpers::ResultType;
use crate::java_type::Type;
#[cfg(feature = "type_check")]
use crate::signature::Signature;

pub fn parameter_to_type(env: &JavaEnv, parameter: &LocalJavaObject) -> ResultType<JavaType> {
    let parameter_class = env.find_class("java/lang/reflect/Parameter")?;
    let get_type = parameter_class.get_object_method("getType", "()Ljava/lang/Class;")?;

    let class_class = env.get_java_lang_class()?;
    let get_name = class_class.get_object_method("getName", "()Ljava/lang/String;")?;

    let parameter_type = get_type
        .call(JavaObject::from(parameter), &[])?
        .ok_or("Parameter.getType() returned null".to_string())?;
    let parameter_name = JavaString::try_from(
        get_name
            .call(JavaObject::from(&parameter_type), &[])?
            .ok_or("Class.getName() returned null".to_string())?,
    )?;

    Ok(JavaType::new(parameter_name.try_into()?, true))
}

pub fn get_method_name(env: &JavaEnv, method: &LocalJavaObject) -> ResultType<String> {
    let method_class = env.find_class("java/lang/reflect/Method")?;
    let get_name = method_class
        .get_object_method("getName", "()Ljava/lang/String;")?
        .bind(method.into());

    let method_name = JavaString::try_from(
        get_name
            .call(&[])?
            .ok_or("Method.getName() returned null".to_string())?,
    )?;
    method_name.try_into()
}

pub fn get_method_return_type(env: &JavaEnv, method: &LocalJavaObject) -> ResultType<JavaType> {
    let method_class = env.find_class("java/lang/reflect/Method")?;
    let get_return_type = method_class
        .get_object_method("getReturnType", "()Ljava/lang/Class;")?
        .bind(method.into());

    let class_class = env.get_java_lang_class()?;
    let get_name = class_class.get_object_method("getName", "()Ljava/lang/String;")?;

    let return_type = get_return_type
        .call(&[])?
        .ok_or("Method.getReturnType() returned null".to_string())?;
    let return_type_name = JavaString::try_from(
        get_name
            .call(JavaObject::from(&return_type), &[])?
            .ok_or("Class.getName() returned null".to_string())?,
    )?;
    Ok(JavaType::new(return_type_name.try_into()?, true))
}

pub fn get_method_parameters(env: &JavaEnv, method: &LocalJavaObject) -> ResultType<Vec<JavaType>> {
    let method_class = env.find_class("java/lang/reflect/Method")?;
    let get_parameters = method_class
        .get_object_method("getParameters", "()[Ljava/lang/reflect/Parameter;")?
        .bind(method.into());
    let parameters = JavaObjectArray::from(
        get_parameters
            .call(&[])?
            .ok_or("Method.getParameters() returned null".to_string())?,
    );

    let num_parameters = parameters.len()?;
    let mut parameter_types: Vec<JavaType> = vec![];
    for i in 0..num_parameters {
        let parameter = parameter_to_type(env, &parameters.get(i)?.ok_or(format!("The array returned by Method.getParameters() contained a null value at position {}", i))?)?;
        parameter_types.push(parameter);
    }
    Ok(parameter_types)
}

pub fn get_method_from_signature(
    env: &JavaEnv,
    class_name: String,
    parameters: &[JavaType],
    return_type: &JavaType,
    name: &str,
    is_static: bool,
) -> ResultType<GlobalJavaMethod> {
    let signature = format!(
        "({}){}",
        parameters
            .iter()
            .map(|p| p.to_jni_type())
            .collect::<Vec<_>>()
            .join(""),
        return_type.to_jni_type()
    );

    let class = JavaClass::by_java_name(class_name.replace('/', "."), env)?;
    let method = if is_static {
        match return_type.type_enum() {
            Type::Void => class.get_static_void_method(name, &signature)?.into(),
            Type::Boolean => class.get_static_boolean_method(name, &signature)?.into(),
            Type::Byte => class.get_static_byte_method(name, &signature)?.into(),
            Type::Character => class.get_static_char_method(name, &signature)?.into(),
            Type::Short => class.get_static_short_method(name, &signature)?.into(),
            Type::Integer => class.get_static_int_method(name, &signature)?.into(),
            Type::Long => class.get_static_long_method(name, &signature)?.into(),
            Type::Float => class.get_static_float_method(name, &signature)?.into(),
            Type::Double => class.get_static_double_method(name, &signature)?.into(),
            _ => class.get_static_object_method(name, &signature)?.into(),
        }
    } else {
        match return_type.type_enum() {
            Type::Void => class.get_void_method(name, &signature)?.into(),
            Type::Boolean => class.get_boolean_method(name, &signature)?.into(),
            Type::Byte => class.get_byte_method(name, &signature)?.into(),
            Type::Character => class.get_char_method(name, &signature)?.into(),
            Type::Short => class.get_short_method(name, &signature)?.into(),
            Type::Integer => class.get_int_method(name, &signature)?.into(),
            Type::Long => class.get_long_method(name, &signature)?.into(),
            Type::Float => class.get_float_method(name, &signature)?.into(),
            Type::Double => class.get_double_method(name, &signature)?.into(),
            _ => class.get_object_method(name, &signature)?.into(),
        }
    };

    let global_class = GlobalJavaClass::by_name(class_name.as_str(), env)?;
    Ok(GlobalJavaMethod::from(global_class, method))
}

pub fn get_constructor_from_signature(
    env: &JavaEnv,
    class_name: String,
    parameters: &[JavaType],
) -> ResultType<GlobalJavaConstructor> {
    let signature = format!(
        "({})V",
        parameters
            .iter()
            .map(|p| p.to_jni_type())
            .collect::<Vec<_>>()
            .join("")
    );

    let class = JavaClass::by_java_name(class_name.replace('/', "."), env)?;
    let constructor = class.get_constructor(signature.as_str())?;

    let global_class = GlobalJavaClass::by_name(class_name.as_str(), env)?;

    Ok(GlobalJavaConstructor::from_local(
        constructor,
        global_class,
        #[cfg(feature = "type_check")]
        Signature::new(JavaType::void(), parameters.to_vec()),
    ))
}

pub fn get_field_type(env: &JavaEnv, field: &LocalJavaObject) -> ResultType<JavaType> {
    let field_class = env.find_class("java/lang/reflect/Field")?;
    let get_type = field_class
        .get_object_method("getType", "()Ljava/lang/Class;")?
        .bind(field.into());

    let class_class = env.get_java_lang_class()?;
    let get_name = class_class.get_object_method("getName", "()Ljava/lang/String;")?;

    let field_type = get_type
        .call(&[])?
        .ok_or("Field.getType() returned null".to_string())?;
    let field_type_name = JavaString::try_from(
        get_name
            .call(JavaObject::from(&field_type), &[])?
            .ok_or("Class.getName() returned null".to_string())?,
    )?;
    Ok(JavaType::new(field_type_name.try_into()?, true))
}

pub fn get_field_from_signature(
    env: &JavaEnv,
    class_name: String,
    field_name: String,
    field_type: JavaType,
    is_static: bool,
) -> ResultType<GlobalJavaField> {
    let class = JavaClass::by_java_name(class_name.replace('/', "."), env)?;
    let field = class.get_field(field_name, field_type, is_static)?;
    let global_class = GlobalJavaClass::by_name(class_name.as_str(), env)?;

    Ok(GlobalJavaField::from(field, global_class))
}
