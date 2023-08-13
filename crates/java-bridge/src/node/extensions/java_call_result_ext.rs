use crate::node::class_cache::ClassCache;
use crate::node::java_class_instance::JavaClassInstance;
use crate::node::util::util::ResultType;
use app_state::{stateful, AppStateTrait, MutAppState};
use java_rs::java_call_result::JavaCallResult;
use java_rs::java_env::JavaEnv;
use java_rs::java_type::{JavaType, Type};
use java_rs::objects::array::{
    JavaArray, JavaBooleanArray, JavaByteArray, JavaCharArray, JavaDoubleArray, JavaFloatArray,
    JavaIntArray, JavaLongArray, JavaObjectArray, JavaShortArray,
};
use java_rs::objects::java_object::JavaObject;
use java_rs::objects::object::{GlobalJavaObject, LocalJavaObject};
use java_rs::traits::IsInstanceOf;
use napi::{Env, JsUnknown};
use std::borrow::Borrow;
use std::sync::{Arc, Mutex};

pub trait ToNapiValue {
    fn to_napi_value(&self, j_env: &JavaEnv, env: &Env) -> ResultType<JsUnknown>;
    fn resolve_object_type(
        &self,
        object: &GlobalJavaObject,
        env: &JavaEnv,
        signature: &JavaType,
        resolve: bool,
    ) -> ResultType<JavaType>;
    fn object_to_napi_value(
        &self,
        object: &GlobalJavaObject,
        j_env: &JavaEnv,
        env: &Env,
        signature: &JavaType,
        objects: bool,
    ) -> ResultType<JsUnknown>;
    fn array_to_napi_value(
        &self,
        object: &GlobalJavaObject,
        j_env: &JavaEnv,
        env: &Env,
        signature: Arc<Mutex<JavaType>>,
    ) -> ResultType<JsUnknown>;
}

impl ToNapiValue for JavaCallResult {
    fn to_napi_value(&self, j_env: &JavaEnv, env: &Env) -> ResultType<JsUnknown> {
        Ok(match self {
            JavaCallResult::Void => env.get_undefined()?.into_unknown(),
            JavaCallResult::Null => env.get_null()?.into_unknown(),
            JavaCallResult::Boolean(b) => env.get_boolean(*b)?.into_unknown(),
            JavaCallResult::Byte(b) => env.create_int32(*b as i32)?.into_unknown(),
            JavaCallResult::Character(c) => env.create_string_utf16(&[*c])?.into_unknown(),
            JavaCallResult::Short(s) => env.create_int32(*s as i32)?.into_unknown(),
            JavaCallResult::Integer(i) => env.create_int32(*i)?.into_unknown(),
            JavaCallResult::Long(l) => env.create_bigint_from_i64(*l)?.into_unknown()?,
            JavaCallResult::Float(f) => env.create_double(*f as f64)?.into_unknown(),
            JavaCallResult::Double(d) => env.create_double(*d)?.into_unknown(),
            JavaCallResult::Object { object, signature } => {
                self.object_to_napi_value(object, j_env, env, signature, true)?
            }
        })
    }

    fn resolve_object_type(
        &self,
        object: &GlobalJavaObject,
        env: &JavaEnv,
        signature: &JavaType,
        resolve: bool,
    ) -> ResultType<JavaType> {
        let res = if object.is_instance_of("java/lang/Integer")? {
            JavaType::integer()
        } else if object.is_instance_of("java/lang/Long")? {
            JavaType::long()
        } else if object.is_instance_of("java/lang/Float")? {
            JavaType::float()
        } else if object.is_instance_of("java/lang/Double")? {
            JavaType::double()
        } else if object.is_instance_of("java/lang/Boolean")? {
            JavaType::boolean()
        } else if object.is_instance_of("java/lang/Byte")? {
            JavaType::byte()
        } else if object.is_instance_of("java/lang/Character")? {
            JavaType::character()
        } else if object.is_instance_of("java/lang/Short")? {
            JavaType::short()
        } else if object.is_instance_of("java/lang/String")? {
            JavaType::string()
        } else if signature.is_array() {
            JavaType::from_existing(
                Some(
                    self.resolve_object_type(
                        object,
                        env,
                        &signature
                            .inner()
                            .ok_or("Expected the signature to have an inner signature".to_string())?
                            .lock()
                            .unwrap()
                            .clone(),
                        true,
                    )?,
                ),
                signature.to_string(),
                Type::Array,
            )
        } else if signature.type_enum() == Type::LangObject && resolve {
            self.resolve_object_type(
                object,
                env,
                &env.get_object_signature(JavaObject::from(object))?,
                false,
            )?
        } else {
            signature.clone()
        };

        Ok(res)
    }

    #[stateful(init(cache))]
    fn object_to_napi_value(
        &self,
        object: &GlobalJavaObject,
        j_env: &JavaEnv,
        env: &Env,
        signature: &JavaType,
        objects: bool,
        cache: MutAppState<ClassCache>,
    ) -> ResultType<JsUnknown> {
        let obj = LocalJavaObject::from(object, &j_env);
        let res = match signature.type_enum() {
            Type::LangInteger => env.create_int32(j_env.object_to_int(&obj)?)?.into_unknown(),
            Type::LangLong => env
                .create_bigint_from_i64(j_env.object_to_long(&obj)?)?
                .into_unknown()?,
            Type::LangShort => env
                .create_int32(j_env.object_to_short(&obj)? as i32)?
                .into_unknown(),
            Type::LangDouble => env
                .create_double(j_env.object_to_double(&obj)?)?
                .into_unknown(),
            Type::LangFloat => env
                .create_double(j_env.object_to_float(&obj)? as f64)?
                .into_unknown(),
            Type::LangByte => env
                .create_int32(j_env.object_to_byte(&obj)? as i32)?
                .into_unknown(),
            Type::LangCharacter => env
                .create_string_utf16(&[j_env.object_to_char(&obj)?])?
                .into_unknown(),
            Type::LangBoolean => env
                .get_boolean(j_env.object_to_boolean(&obj)?)?
                .into_unknown(),
            Type::String => {
                let str = j_env.object_to_string(&obj)?;
                env.create_string_from_std(str)?.into_unknown()
            }
            Type::Array => self.array_to_napi_value(
                object,
                j_env,
                env,
                signature.inner().ok_or("No inner type provided")?,
            )?,
            Type::LangObject | Type::Object => {
                if objects {
                    self.object_to_napi_value(
                        object,
                        j_env,
                        env,
                        &self.resolve_object_type(object, j_env, signature, true)?,
                        false,
                    )?
                } else {
                    let vm = j_env.get_java_vm()?;
                    let proxy = cache
                        .get_mut()
                        .get_class_proxy(&vm, signature.to_string(), None)?;

                    JavaClassInstance::from_existing(proxy, env, object.clone())?
                }
            }
            _ => {
                return Err(format!("Invalid object type: {}", signature).into());
            }
        };

        Ok(res)
    }

    fn array_to_napi_value(
        &self,
        object: &GlobalJavaObject,
        j_env: &JavaEnv,
        env: &Env,
        signature: Arc<Mutex<JavaType>>,
    ) -> ResultType<JsUnknown> {
        let obj = LocalJavaObject::from(object, &j_env);
        let arr = JavaArray::from(obj);

        let mut res = env.create_array(arr.len()? as u32)?;
        let sig = signature.lock().unwrap();

        match sig.type_enum() {
            Type::Integer => {
                let data = JavaIntArray::from(arr).get_data()?;
                for i in 0..data.len() {
                    res.set(i as u32, data[i])?;
                }
            }
            Type::Long => {
                let data = JavaLongArray::from(arr).get_data()?;
                for i in 0..data.len() {
                    res.set(i as u32, data[i])?;
                }
            }
            Type::Short => {
                let data = JavaShortArray::from(arr).get_data()?;
                for i in 0..data.len() {
                    res.set(i as u32, data[i])?;
                }
            }
            Type::Double => {
                let data = JavaDoubleArray::from(arr).get_data()?;
                for i in 0..data.len() {
                    res.set(i as u32, data[i])?;
                }
            }
            Type::Float => {
                let data = JavaFloatArray::from(arr).get_data()?;
                for i in 0..data.len() {
                    res.set(i as u32, data[i] as f64)?;
                }
            }
            Type::Boolean => {
                let data = JavaBooleanArray::from(arr).get_data()?;
                for i in 0..data.len() {
                    res.set(i as u32, data[i] != 0)?;
                }
            }
            Type::Character => {
                let data = JavaCharArray::from(arr).get_data()?;
                for i in 0..data.len() {
                    res.set(i as u32, env.create_string_utf16(&[data[i]]))?;
                }
            }
            Type::Byte => {
                let data = JavaByteArray::from(arr)
                    .get_data()?
                    .iter()
                    .map(|b| *b as u8)
                    .collect();
                return Ok(env.create_buffer_with_data(data)?.into_unknown());
            }
            Type::LangByte => {
                let array = JavaObjectArray::from(arr);
                let mut data: Vec<i8> = Vec::with_capacity(array.len()? as usize);

                for i in 0..array.len()? {
                    data.push(if let Some(obj) = array.get(i)? {
                        j_env.object_to_byte(&obj)?
                    } else {
                        0
                    });
                }

                let data = data.iter().map(|b| *b as u8).collect();
                return Ok(env.create_buffer_with_data(data)?.into_unknown());
            }
            _ => {
                let array = JavaObjectArray::from(arr);
                for i in 0..array.len()? {
                    res.set(
                        i as u32,
                        if let Some(obj) = array.get(i)? {
                            let obj = GlobalJavaObject::try_from(obj)?;
                            self.object_to_napi_value(&obj, j_env, env, sig.clone().borrow(), true)?
                        } else {
                            env.get_null()?.into_unknown()
                        },
                    )?;
                }
            }
        };

        Ok(res.coerce_to_object()?.into_unknown())
    }
}
