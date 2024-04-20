use crate::node::util::helpers::ResultType;
use java_rs::java_env::JavaEnv;
use java_rs::java_type::JavaType;
use java_rs::java_vm::JavaVM;
use java_rs::objects::args::JavaArgs;
use java_rs::objects::array::JavaObjectArray;
use java_rs::objects::class::{GlobalJavaClass, JavaClass};
use java_rs::objects::constructor::{GlobalJavaConstructor, JavaConstructor};
use java_rs::objects::java_object::JavaObject;
use java_rs::objects::object::GlobalJavaObject;
use java_rs::util::conversion::{get_constructor_from_signature, parameter_to_type};
use std::fmt::Display;

#[derive(Clone)]
pub struct ClassConstructor {
    vm: JavaVM,
    parameter_types: Vec<JavaType>,
    constructor: GlobalJavaConstructor,
    class_name: String,
}

impl ClassConstructor {
    pub fn get_constructors(vm: JavaVM, class_name: String) -> ResultType<Vec<Self>> {
        let env = vm.attach_thread()?;
        let class = GlobalJavaClass::by_name(class_name.as_str(), &env)?;
        let java_class = env.get_java_lang_class()?;
        let local_class = JavaClass::from_global(&class, &env);

        let get_constructors = java_class
            .get_object_method("getConstructors", "()[Ljava/lang/reflect/Constructor;")?
            .bind(JavaObject::from(local_class.to_object()));
        let constructors = JavaObjectArray::from(
            get_constructors
                .call(&[])?
                .ok_or("Class.getConstructors() returned null".to_string())?,
        );

        let num_constructors = constructors.len()?;
        let mut res: Vec<ClassConstructor> = vec![];

        for i in 0..num_constructors {
            let constructor = constructors.get(i)?.ok_or(
                "A value in the array returned by Class.getConstructors() was null".to_string(),
            )?;
            res.push(ClassConstructor::new(
                vm.clone(),
                &env,
                class_name.clone(),
                GlobalJavaObject::try_from(constructor)?,
            )?);
        }

        Ok(res)
    }

    fn new<'a>(
        vm: JavaVM,
        env: &'a JavaEnv<'a>,
        class_name: String,
        constructor: GlobalJavaObject,
    ) -> ResultType<Self> {
        let constructor_class = env.find_class("java/lang/reflect/Constructor")?;
        let get_parameters = constructor_class
            .get_object_method("getParameters", "()[Ljava/lang/reflect/Parameter;")?
            .bind(JavaObject::from(constructor.clone()));

        let parameters = JavaObjectArray::from(
            get_parameters
                .call(&[])?
                .ok_or("Constructor.getParameters() returned null".to_string())?,
        );
        let num_parameters = parameters.len()?;

        let mut parameter_types: Vec<JavaType> = vec![];
        for i in 0..num_parameters {
            let parameter = parameter_to_type(
                env,
                &parameters.get(i)?.ok_or(
                    "A value in the array returned by Constructor.getParameters() was null"
                        .to_string(),
                )?,
            )?;

            parameter_types.push(parameter);
        }

        Ok(Self {
            vm,
            parameter_types: parameter_types.clone(),
            class_name: class_name.clone(),
            constructor: get_constructor_from_signature(env, class_name, &parameter_types)?,
        })
    }

    pub fn new_instance(&self, args: JavaArgs) -> ResultType<GlobalJavaObject> {
        let env = self.vm.attach_thread()?;
        let class = self.constructor.get_class(&env);

        let constructor = JavaConstructor::from_global(&self.constructor, &class);
        let instance = constructor.new_instance(&env, args)?;

        GlobalJavaObject::try_from(instance)
    }

    pub fn parameter_types(&self) -> &Vec<JavaType> {
        &self.parameter_types
    }
}

impl Display for ClassConstructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.class_name,
            self.parameter_types
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
