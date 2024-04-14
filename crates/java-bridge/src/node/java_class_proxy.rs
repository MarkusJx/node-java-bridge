use crate::java::class_constructor::ClassConstructor;
use crate::java::class_field::ClassField;
use crate::java::class_method::ClassMethod;
use crate::node::config::Config;
use crate::node::extensions::class_ext::ArgumentMatch;
use java_rs::java_vm::JavaVM;
use java_rs::objects::class::GlobalJavaClass;
use java_rs::util::helpers::ResultType;
use napi::CallContext;
use std::collections::HashMap;

pub struct JavaClassProxy {
    pub vm: JavaVM,
    pub class: GlobalJavaClass,
    pub methods: HashMap<String, Vec<ClassMethod>>,
    pub static_methods: HashMap<String, Vec<ClassMethod>>,
    pub fields: HashMap<String, ClassField>,
    pub static_fields: HashMap<String, ClassField>,
    pub constructors: Vec<ClassConstructor>,
    pub class_name: String,
    pub config: Config,
}

impl JavaClassProxy {
    pub fn new(vm: JavaVM, class_name: String, config: Option<Config>) -> ResultType<Self> {
        let env = vm.attach_thread()?;
        let class = env.find_global_class_by_java_name(class_name.clone())?;

        Ok(Self {
            vm: vm.clone(),
            class,
            methods: ClassMethod::get_all_methods(vm.clone(), class_name.clone(), false)?,
            static_methods: ClassMethod::get_all_methods(vm.clone(), class_name.clone(), true)?,
            fields: ClassField::get_class_fields(vm.clone(), class_name.clone(), false)?,
            static_fields: ClassField::get_class_fields(vm.clone(), class_name.clone(), true)?,
            constructors: ClassConstructor::get_constructors(vm, class_name.clone())?,
            class_name,
            config: config.unwrap_or_else(|| Config::get().clone()),
        })
    }

    pub fn find_matching_method(
        &self,
        ctx: &CallContext,
        name: &String,
        only_static: bool,
        allow_object: bool,
    ) -> ResultType<&ClassMethod> {
        let methods = if only_static {
            &self.static_methods
        } else {
            &self.methods
        }
        .get(name)
        .ok_or(format!("No method found with name '{}'", name))?;

        methods
            .iter()
            .map(|m| {
                m.arguments_match(ctx, allow_object)
                    .map(|r| if r { Some(m) } else { None })
            })
            .collect::<napi::Result<Vec<Option<&ClassMethod>>>>()?
            .iter()
            .flatten()
            .copied()
            .next()
            .ok_or(
                format!(
                    "No method found with name '{}' and matching signature. Options were:\n{}",
                    name,
                    methods
                        .iter()
                        .map(|m| {
                            let static_prefix = if only_static { "static " } else { "" };
                            format!("\tpublic {}{}", static_prefix, m)
                        })
                        .collect::<Vec<String>>()
                        .join("\n")
                )
                .into(),
            )
    }

    pub fn find_matching_constructor(
        &self,
        ctx: &CallContext,
        allow_object: bool,
    ) -> ResultType<&ClassConstructor> {
        self.constructors
            .iter()
            .map(|m| {
                m.arguments_match(ctx, allow_object)
                    .map(|r| if r { Some(m) } else { None })
            })
            .collect::<napi::Result<Vec<Option<&ClassConstructor>>>>()?
            .iter()
            .flatten()
            .copied()
            .next()
            .ok_or(
                format!(
                    "No constructor found with matching signature. Options were:\n{}",
                    self.constructors
                        .iter()
                        .map(|c| format!("\tpublic {}", c))
                        .collect::<Vec<String>>()
                        .join("\n")
                )
                .into(),
            )
    }

    pub fn get_field_by_name(&self, name: &str) -> ResultType<&ClassField> {
        self.fields
            .get(name)
            .ok_or(format!("No field found with name '{}'", name).into())
    }

    pub fn get_static_field_by_name(&self, name: &str) -> ResultType<&ClassField> {
        self.static_fields
            .get(name)
            .ok_or(format!("No static field found with name '{}'", name).into())
    }
}

unsafe impl Send for JavaClassProxy {}
