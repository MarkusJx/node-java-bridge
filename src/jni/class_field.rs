use crate::jni::java_call_result::JavaCallResult;
use crate::jni::java_env::JavaEnv;
use crate::jni::java_field::GlobalJavaField;
use crate::jni::java_type::JavaType;
use crate::jni::java_vm::JavaVM;
use crate::jni::objects::array::JavaObjectArray;
use crate::jni::objects::class::JavaClass;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::object::{GlobalJavaObject, LocalJavaObject};
use crate::jni::objects::string::JavaString;
use crate::jni::util::conversion::{get_field_from_signature, get_field_type};
use crate::jni::util::util::{field_is_final, method_is_public, ResultType};
use std::collections::HashMap;

pub struct ClassField {
    vm: JavaVM,
    name: String,
    field: GlobalJavaField,
    is_final: bool,
}

impl ClassField {
    pub fn get_class_fields(
        vm: JavaVM,
        class_name: String,
        only_static: bool,
    ) -> ResultType<HashMap<String, Self>> {
        let env = vm.attach_thread()?;
        let class = env.find_global_class_by_java_name(class_name.clone())?;
        let java_class = env.get_java_lang_class()?;
        let get_declared_fields = java_class
            .get_object_method("getDeclaredFields", "()[Ljava/lang/reflect/Field;")?
            .bind(JavaObject::from(class));

        let field = JavaClass::by_name("java/lang/reflect/Field", &env)?;
        let get_name = field.get_object_method("getName", "()Ljava/lang/String;")?;

        let fields = JavaObjectArray::from(get_declared_fields.call(vec![])?);
        let num_fields = fields.len()?;

        let mut res: HashMap<String, Self> = HashMap::new();
        for i in 0..num_fields {
            let field = fields.get(i)?;

            if method_is_public(&env, &field, false, only_static)? {
                let name = JavaString::from(get_name.call(JavaObject::from(&field), vec![])?)
                    .to_string()?;

                let class_field = ClassField::from_field(
                    vm.clone(),
                    &env,
                    name.clone(),
                    class_name.clone(),
                    field,
                    only_static,
                )?;

                res.insert(name, class_field);
            }
        }

        Ok(res)
    }

    fn from_field(
        vm: JavaVM,
        env: &JavaEnv,
        name: String,
        class_name: String,
        field: LocalJavaObject,
        is_static: bool,
    ) -> ResultType<Self> {
        let field_type = get_field_type(&env, &field)?;
        Ok(ClassField {
            vm,
            name: name.clone(),
            field: get_field_from_signature(&env, class_name, name, field_type, is_static)?,
            is_final: field_is_final(&env, &field)?,
        })
    }

    pub fn set(&self, object: &GlobalJavaObject, value: JavaCallResult) -> ResultType<()> {
        if self.is_final {
            return Err(format!("Field {} is final", self.name).into());
        }

        let env = self.vm.attach_thread()?;
        let class = self.field.get_class(&env);

        let field = self.field.to_java_field(&class)?;
        field.set(&JavaObject::from(object), value)
    }

    pub fn get(&self, object: &GlobalJavaObject) -> ResultType<JavaCallResult> {
        let env = self.vm.attach_thread()?;
        let class = self.field.get_class(&env);

        let field = self.field.to_java_field(&class)?;
        field.get(&JavaObject::from(object))
    }

    pub fn set_static(&self, value: JavaCallResult) -> ResultType<()> {
        if self.is_final {
            return Err(format!("Field {} is final", self.name).into());
        }

        let env = self.vm.attach_thread()?;
        let class = self.field.get_class(&env);

        let field = self.field.to_static_java_field(&class)?;
        field.set(value)
    }

    pub fn get_static(&self) -> ResultType<JavaCallResult> {
        let env = self.vm.attach_thread()?;
        let class = self.field.get_class(&env);

        let field = self.field.to_static_java_field(&class)?;
        field.get()
    }

    pub fn get_type(&self) -> &JavaType {
        self.field.get_type()
    }

    pub fn is_final(&self) -> bool {
        self.is_final
    }
}

unsafe impl Send for ClassField {}
