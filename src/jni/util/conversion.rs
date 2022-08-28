use crate::jni::java_env::JavaEnv;
use crate::jni::java_field::GlobalJavaField;
use crate::jni::java_type::JavaType;
use crate::jni::objects::array::JavaObjectArray;
use crate::jni::objects::class::{GlobalJavaClass, JavaClass};
use crate::jni::objects::constructor::GlobalJavaConstructor;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::method::GlobalJavaMethod;
use crate::jni::objects::object::LocalJavaObject;
use crate::jni::objects::string::JavaString;
use crate::jni::util::util::ResultType;

pub fn parameter_to_type(env: &JavaEnv, parameter: &LocalJavaObject) -> ResultType<JavaType> {
    let parameter_class = env.find_class("java/lang/reflect/Parameter")?;
    let get_type = parameter_class.get_object_method("getType", "()Ljava/lang/Class;")?;

    let class_class = env.get_java_lang_class()?;
    let get_name = class_class.get_object_method("getName", "()Ljava/lang/String;")?;

    let parameter_type = get_type.call(JavaObject::from(parameter), vec![])?;
    let parameter_name =
        JavaString::from(get_name.call(JavaObject::from(&parameter_type), vec![])?);

    Ok(JavaType::new(parameter_name.try_into()?, true))
}

pub fn get_method_name(env: &JavaEnv, method: &LocalJavaObject) -> ResultType<String> {
    let method_class = env.find_class("java/lang/reflect/Method")?;
    let get_name = method_class
        .get_object_method("getName", "()Ljava/lang/String;")?
        .bind(method.into());

    let method_name = JavaString::from(get_name.call(vec![])?);
    Ok(method_name.try_into()?)
}

pub fn get_method_return_type(env: &JavaEnv, method: &LocalJavaObject) -> ResultType<JavaType> {
    let method_class = env.find_class("java/lang/reflect/Method")?;
    let get_return_type = method_class
        .get_object_method("getReturnType", "()Ljava/lang/Class;")?
        .bind(method.into());

    let class_class = env.get_java_lang_class()?;
    let get_name = class_class.get_object_method("getName", "()Ljava/lang/String;")?;

    let return_type = get_return_type.call(vec![])?;
    let return_type_name = JavaString::from(get_name.call(JavaObject::from(&return_type), vec![])?);
    Ok(JavaType::new(return_type_name.try_into()?, true))
}

pub fn get_method_parameters(env: &JavaEnv, method: &LocalJavaObject) -> ResultType<Vec<JavaType>> {
    let method_class = env.find_class("java/lang/reflect/Method")?;
    let get_parameters = method_class
        .get_object_method("getParameters", "()[Ljava/lang/reflect/Parameter;")?
        .bind(method.into());
    let parameters = JavaObjectArray::from(get_parameters.call(vec![])?);

    let num_parameters = parameters.len()?;
    let mut parameter_types: Vec<JavaType> = vec![];
    for i in 0..num_parameters {
        let parameter = parameter_to_type(env, &parameters.get(i)?)?;
        parameter_types.push(parameter);
    }
    Ok(parameter_types)
}

pub fn get_method_from_signature(
    env: &JavaEnv,
    class_name: String,
    parameters: &Vec<JavaType>,
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

    let class = JavaClass::by_name(class_name.replace('.', "/").as_str(), env)?;
    let method = if is_static {
        class
            .get_static_object_method(name, signature.as_str())?
            .into()
    } else {
        class.get_object_method(name, signature.as_str())?.into()
    };

    let global_class = GlobalJavaClass::by_name(class_name.as_str(), env)?;
    Ok(GlobalJavaMethod::from(global_class, method))
}

pub fn get_constructor_from_signature(
    env: &JavaEnv,
    class_name: String,
    parameters: &Vec<JavaType>,
) -> ResultType<GlobalJavaConstructor> {
    let signature = format!(
        "({})V",
        parameters
            .iter()
            .map(|p| p.to_jni_type())
            .collect::<Vec<_>>()
            .join("")
    );

    let class = JavaClass::by_name(class_name.replace('.', "/").as_str(), env)?;
    let constructor = class.get_constructor(signature.as_str())?;

    let global_class = GlobalJavaClass::by_name(class_name.as_str(), env)?;
    Ok(GlobalJavaConstructor::from_local(constructor, global_class))
}

pub fn get_field_type(env: &JavaEnv, field: &LocalJavaObject) -> ResultType<JavaType> {
    let field_class = env.find_class("java/lang/reflect/Field")?;
    let get_type = field_class
        .get_object_method("getType", "()Ljava/lang/Class;")?
        .bind(field.into());

    let class_class = env.get_java_lang_class()?;
    let get_name = class_class.get_object_method("getName", "()Ljava/lang/String;")?;

    let field_type = get_type.call(vec![])?;
    let field_type_name = JavaString::from(get_name.call(JavaObject::from(&field_type), vec![])?);
    Ok(JavaType::new(field_type_name.try_into()?, true))
}

pub fn get_field_from_signature(
    env: &JavaEnv,
    class_name: String,
    field_name: String,
    field_type: JavaType,
    is_static: bool,
) -> ResultType<GlobalJavaField> {
    let class = JavaClass::by_name(class_name.replace('.', "/").as_str(), env)?;
    let field = class.get_field(field_name, field_type, is_static)?;
    let global_class = GlobalJavaClass::by_name(class_name.as_str(), env)?;

    Ok(GlobalJavaField::from(field, global_class))
}
