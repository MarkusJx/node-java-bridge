use crate::jni::java_call_result::JavaCallResult;
use crate::jni::java_env::JavaEnv;
use crate::jni::java_type::{JavaType, Type};
use crate::jni::java_vm::JavaVM;
use crate::jni::objects::args::JavaArgs;
use crate::jni::objects::array::JavaObjectArray;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::method::{
    GlobalJavaMethod, JavaBooleanMethod, JavaByteMethod, JavaCharMethod, JavaDoubleMethod,
    JavaFloatMethod, JavaIntMethod, JavaLongMethod, JavaObjectMethod, JavaShortMethod,
    JavaVoidMethod, StaticJavaBooleanMethod, StaticJavaByteMethod, StaticJavaCharMethod,
    StaticJavaDoubleMethod, StaticJavaFloatMethod, StaticJavaIntMethod, StaticJavaLongMethod,
    StaticJavaObjectMethod, StaticJavaShortMethod, StaticJavaVoidMethod,
};
use crate::jni::objects::object::{GlobalJavaObject, LocalJavaObject};
use crate::jni::traits::IsNull;
use crate::jni::util::conversion::{
    get_method_from_signature, get_method_name, get_method_parameters, get_method_return_type,
};
use crate::jni::util::util::{method_is_public, ResultType};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct ClassMethod {
    vm: JavaVM,
    parameter_types: Vec<JavaType>,
    return_type: JavaType,
    method: GlobalJavaMethod,
    name: String,
    is_static: bool,
}

impl ClassMethod {
    pub fn get_all_methods(
        vm: JavaVM,
        class_name: String,
        only_static: bool,
    ) -> ResultType<HashMap<String, Vec<Self>>> {
        let env = vm.attach_thread()?;
        let class = env.find_global_class_by_java_name(class_name.clone())?;
        let java_class = env.get_java_lang_class()?;
        let get_methods = java_class
            .get_object_method("getMethods", "()[Ljava/lang/reflect/Method;")?
            .bind(JavaObject::from(class));
        let methods = JavaObjectArray::from(get_methods.call(vec![])?);
        let num_methods = methods.len()?;

        let mut res: HashMap<String, Vec<Self>> = HashMap::new();
        for i in 0..num_methods {
            let method = methods.get(i)?;

            if method_is_public(&env, &method, true, only_static)? {
                let method = ClassMethod::from_method(
                    vm.clone(),
                    &env,
                    method,
                    class_name.clone(),
                    only_static,
                )?;

                let method_name = method.name.clone();
                res.entry(method_name).or_insert(vec![]).push(method);
            }
        }
        Ok(res)
    }

    pub(crate) fn parameter_types(&self) -> &Vec<JavaType> {
        &self.parameter_types
    }

    fn from_method(
        vm: JavaVM,
        env: &JavaEnv,
        method: LocalJavaObject,
        class_name: String,
        is_static: bool,
    ) -> ResultType<Self> {
        let parameter_types = get_method_parameters(env, &method)?;
        let return_type = get_method_return_type(env, &method)?;
        let name = get_method_name(env, &method)?;

        Ok(Self {
            vm,
            method: get_method_from_signature(
                env,
                class_name,
                &parameter_types,
                &return_type,
                name.as_str(),
                is_static,
            )?,
            name,
            parameter_types,
            return_type,
            is_static,
        })
    }

    pub fn call<'a>(
        &self,
        object: &'a GlobalJavaObject,
        args: JavaArgs<'a>,
    ) -> ResultType<JavaCallResult> {
        let env = self.vm.attach_thread()?;
        if self.is_static {
            return Err("Tried calling static method non-statically".into());
        }

        let class = self.method.get_class(&env);
        let res = match self.return_type.type_enum() {
            Type::Void => {
                JavaVoidMethod::from_global(self.method.clone(), &class)?
                    .bind(JavaObject::from(object))
                    .call(args)?;

                JavaCallResult::Void
            }
            Type::Boolean => {
                let res = JavaBooleanMethod::from_global(self.method.clone(), &class)?
                    .bind(JavaObject::from(object))
                    .call(args)?;
                JavaCallResult::Boolean(res)
            }
            Type::Byte => {
                let res = JavaByteMethod::from_global(self.method.clone(), &class)?
                    .bind(JavaObject::from(object))
                    .call(args)?;
                JavaCallResult::Byte(res)
            }
            Type::Character => {
                let res = JavaCharMethod::from_global(self.method.clone(), &class)?
                    .bind(JavaObject::from(object))
                    .call(args)?;
                JavaCallResult::Character(res)
            }
            Type::Short => {
                let res = JavaShortMethod::from_global(self.method.clone(), &class)?
                    .bind(JavaObject::from(object))
                    .call(args)?;
                JavaCallResult::Short(res)
            }
            Type::Integer => {
                let res = JavaIntMethod::from_global(self.method.clone(), &class)?
                    .bind(JavaObject::from(object))
                    .call(args)?;
                JavaCallResult::Integer(res)
            }
            Type::Long => {
                let res = JavaLongMethod::from_global(self.method.clone(), &class)?
                    .bind(JavaObject::from(object))
                    .call(args)?;
                JavaCallResult::Long(res)
            }
            Type::Float => {
                let res = JavaFloatMethod::from_global(self.method.clone(), &class)?
                    .bind(JavaObject::from(object))
                    .call(args)?;
                JavaCallResult::Float(res)
            }
            Type::Double => {
                let res = JavaDoubleMethod::from_global(self.method.clone(), &class)?
                    .bind(JavaObject::from(object))
                    .call(args)?;
                JavaCallResult::Double(res)
            }
            _ => {
                let m = JavaObjectMethod::from_global(self.method.clone(), &class)?
                    .bind(JavaObject::from(object));
                let res = m.call(args)?;

                if res.is_null() {
                    JavaCallResult::Null
                } else {
                    JavaCallResult::Object {
                        object: GlobalJavaObject::try_from(res)?,
                        signature: self.return_type.clone(),
                    }
                }
            }
        };

        Ok(res)
    }

    pub fn call_static(&self, args: JavaArgs) -> ResultType<JavaCallResult> {
        let env = self.vm.attach_thread()?;
        if !self.is_static {
            return Err("Tried calling non-static method statically".into());
        }

        let class = self.method.get_class(&env);
        Ok(match self.return_type.type_enum() {
            Type::Void => {
                StaticJavaVoidMethod::from_global(self.method.clone(), &class)?.call(args)?;
                JavaCallResult::Void
            }
            Type::Boolean => {
                let res = StaticJavaBooleanMethod::from_global(self.method.clone(), &class)?
                    .call(args)?;
                JavaCallResult::Boolean(res)
            }
            Type::Byte => {
                let res =
                    StaticJavaByteMethod::from_global(self.method.clone(), &class)?.call(args)?;
                JavaCallResult::Byte(res)
            }
            Type::Character => {
                let res =
                    StaticJavaCharMethod::from_global(self.method.clone(), &class)?.call(args)?;
                JavaCallResult::Character(res)
            }
            Type::Short => {
                let res =
                    StaticJavaShortMethod::from_global(self.method.clone(), &class)?.call(args)?;
                JavaCallResult::Short(res)
            }
            Type::Integer => {
                let res =
                    StaticJavaIntMethod::from_global(self.method.clone(), &class)?.call(args)?;
                JavaCallResult::Integer(res)
            }
            Type::Long => {
                let res =
                    StaticJavaLongMethod::from_global(self.method.clone(), &class)?.call(args)?;
                JavaCallResult::Long(res)
            }
            Type::Float => {
                let res =
                    StaticJavaFloatMethod::from_global(self.method.clone(), &class)?.call(args)?;
                JavaCallResult::Float(res)
            }
            Type::Double => {
                let res =
                    StaticJavaDoubleMethod::from_global(self.method.clone(), &class)?.call(args)?;
                JavaCallResult::Double(res)
            }
            _ => {
                let m = StaticJavaObjectMethod::from_global(self.method.clone(), &class)?;
                let res = m.call(args)?;
                JavaCallResult::Object {
                    object: GlobalJavaObject::try_from(res)?,
                    signature: self.return_type.clone(),
                }
            }
        })
    }
}

impl Display for ClassMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}({})",
            self.return_type,
            self.name,
            self.parameter_types
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
