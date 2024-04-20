use crate::node::helpers::js_to_java_object::{JsIntoJavaObject, JsToJavaClass};
use crate::node::helpers::napi_error::{MapToNapiError, NapiError};
use crate::node::interface_proxy::java_interface_proxy::JavaInterfaceProxy;
use crate::node::java_class_instance::OBJECT_PROPERTY;
use java_rs::java_call_result::JavaCallResult;
use java_rs::java_env::JavaEnv;
use java_rs::java_type::{JavaType, Type};
use java_rs::objects::args::JavaArg;
use java_rs::objects::array::{
    JavaBooleanArray, JavaByteArray, JavaCharArray, JavaDoubleArray, JavaFloatArray, JavaIntArray,
    JavaLongArray, JavaObjectArray, JavaShortArray,
};
use java_rs::objects::class::JavaClass;
use java_rs::objects::java_object::JavaObject;
use java_rs::objects::object::{GlobalJavaObject, LocalJavaObject};
use java_rs::objects::string::JavaString;
use java_rs::traits::GetSignature;
use java_rs::util::helpers::ResultType;
use napi::{
    Env, JsBigInt, JsBoolean, JsBuffer, JsFunction, JsNumber, JsObject, JsString, JsTypedArray,
    JsUnknown, ValueType,
};
use std::ops::Deref;

fn value_may_be_byte(value: JsUnknown) -> napi::Result<bool> {
    let val = value.coerce_to_number()?.get_double().unwrap_or(-1.0);
    Ok((-128.0..=127.0).contains(&val) && val.round() == val)
}

fn is_integer(env: &Env, value: &JsNumber) -> ResultType<bool> {
    let number = env
        .get_global()?
        .get_named_property::<JsFunction>("Number")?
        .coerce_to_object()?;
    let is_integer: JsFunction = number.get_named_property("isInteger")?;

    Ok(is_integer
        .call(None, &[value])?
        .coerce_to_bool()?
        .get_value()?)
}

pub trait JsTypeEq {
    fn js_equals(&self, other: JsUnknown, env: &Env) -> napi::Result<bool>;
}

impl JsTypeEq for JavaType {
    fn js_equals(&self, other: JsUnknown, env: &Env) -> napi::Result<bool> {
        Ok(match other.get_type()? {
            ValueType::String => {
                Type::String == self
                    || Type::CharSequence == self
                    || (other.coerce_to_string()?.utf16_len()? == 1 && self.is_char())
            }
            ValueType::Number => {
                self.is_int()
                    || self.is_double()
                    || self.is_float()
                    || self.is_long()
                    || (value_may_be_byte(other)? && self.is_byte())
            }
            ValueType::Boolean => self.is_boolean(),
            ValueType::BigInt => self.is_long(),
            ValueType::Null | ValueType::Undefined => !self.is_primitive(),
            ValueType::Object => {
                if (other.is_array()? || other.is_typedarray()?)
                    && self.is_array()
                    && !other.is_buffer()?
                {
                    let arr = unsafe { other.cast::<JsTypedArray>() };
                    if arr.get_array_length()? == 0 {
                        true
                    } else {
                        self.inner()
                            .unwrap()
                            .lock()
                            .unwrap()
                            .deref()
                            .js_equals(arr.get_element(0)?, env)?
                    }
                } else if other.is_buffer()? {
                    self.is_byte_array()
                } else if JavaInterfaceProxy::instance_of(*env, &other)? {
                    let proxy: JsUnknown = other.coerce_to_object()?.get_named_property("proxy")?;
                    if proxy.get_type()? == ValueType::Null {
                        return Err(
                            NapiError::from("The proxy has already been destroyed").into_napi()
                        );
                    }

                    let obj = env.unwrap::<GlobalJavaObject>(&proxy.coerce_to_object()?)?;
                    let j_env = obj.get_vm().attach_thread().map_napi_err(Some(*env))?;
                    let other_class = obj.get_class(&j_env).map_napi_err(Some(*env))?;

                    let self_class = self.as_class(&j_env).map_napi_err(Some(*env))?;
                    self_class
                        .is_assignable_from(&other_class)
                        .map_napi_err(Some(*env))?
                } else if !self.is_array()
                    && !other.is_promise()?
                    && !other.is_date()?
                    && !other.is_error()?
                    && !other.is_dataview()?
                {
                    let object = other.coerce_to_object();
                    if object.is_err() {
                        return Ok(false);
                    }

                    let object = object.unwrap();
                    let class = object.to_java_class(env);
                    if class.is_err() {
                        return Ok(false);
                    }

                    let class = class.unwrap();
                    let j_env = class.vm.attach_thread().map_napi_err(Some(*env))?;
                    let self_class = self.as_class(&j_env).map_napi_err(Some(*env))?;
                    let other_class = JavaClass::from_global(&class.class, &j_env);
                    self_class
                        .is_assignable_from(&other_class)
                        .map_napi_err(Some(*env))?
                } else {
                    false
                }
            }
            // other is a function, a symbol or unknown
            _ => false,
        })
    }
}

pub trait JavaToNapi {
    fn to_napi(&self, value: &JavaArg) -> ResultType<JsUnknown>;
}

pub trait NapiToJava {
    fn convert_to_java_object<'a>(
        &self,
        env: &'a JavaEnv,
        node_env: &'a Env,
        value: JsUnknown,
    ) -> ResultType<Option<JavaObject<'a>>>;

    fn convert_to_java_value<'a>(
        &self,
        env: &'a JavaEnv<'a>,
        node_env: &'a Env,
        value: JsUnknown,
    ) -> ResultType<JavaCallResult>;

    fn array_to_java_value<'a>(
        &self,
        env: &'a JavaEnv<'a>,
        node_env: &'a Env,
        value: JsUnknown,
    ) -> ResultType<JavaObject<'a>>;
}

impl NapiToJava for JavaType {
    fn convert_to_java_object<'a>(
        &self,
        env: &'a JavaEnv,
        node_env: &'a Env,
        value: JsUnknown,
    ) -> ResultType<Option<JavaObject<'a>>> {
        Ok(Some(match self.type_enum() {
            Type::LangInteger | Type::Integer => {
                if value.get_type()? == ValueType::Object {
                    JavaObject::from(value.into_java_object(node_env)?)
                } else {
                    let val = value.coerce_to_number()?.get_int32()?;
                    JavaObject::from(LocalJavaObject::from_i32(env, val)?)
                }
            }
            Type::LangLong | Type::Long => {
                if value.get_type()? == ValueType::Object {
                    JavaObject::from(value.into_java_object(node_env)?)
                } else {
                    let val = if value.get_type()? == ValueType::BigInt {
                        unsafe { value.cast::<JsBigInt>() }.get_i64()?.0
                    } else {
                        value.coerce_to_number()?.get_int64()?
                    };

                    LocalJavaObject::from_i64(env, val)?.into()
                }
            }
            Type::LangFloat | Type::Float => {
                if value.get_type()? == ValueType::Object {
                    JavaObject::from(value.into_java_object(node_env)?)
                } else {
                    let val = value.coerce_to_number()?.get_double()?;
                    LocalJavaObject::from_f32(env, val as f32)?.into()
                }
            }
            Type::LangDouble | Type::Double => {
                if value.get_type()? == ValueType::Object {
                    JavaObject::from(value.into_java_object(node_env)?)
                } else {
                    let val = value.coerce_to_number()?.get_double()?;
                    LocalJavaObject::from_f64(env, val)?.into()
                }
            }
            Type::LangBoolean | Type::Boolean => {
                if value.get_type()? == ValueType::Object {
                    JavaObject::from(value.into_java_object(node_env)?)
                } else {
                    let val = value.coerce_to_bool()?.get_value()?;
                    LocalJavaObject::from_bool(env, val)?.into()
                }
            }
            Type::LangCharacter | Type::Character => {
                if value.get_type()? == ValueType::Object {
                    JavaObject::from(value.into_java_object(node_env)?)
                } else {
                    let val = value.coerce_to_string()?.into_utf16()?;
                    let slice = val.as_slice();

                    if slice.len() == 1 {
                        LocalJavaObject::from_char(env, slice[0])?.into()
                    } else {
                        return Err("Java character must be a single character".into());
                    }
                }
            }
            Type::LangByte | Type::Byte => {
                if value.get_type()? == ValueType::Object {
                    JavaObject::from(value.into_java_object(node_env)?)
                } else {
                    let val = value.coerce_to_number()?.get_int32()?;
                    LocalJavaObject::from_byte(env, val as i8)?.into()
                }
            }
            Type::LangShort | Type::Short => {
                if value.get_type()? == ValueType::Object {
                    JavaObject::from(value.into_java_object(node_env)?)
                } else {
                    let val = value.coerce_to_number()?.get_int32()?;
                    LocalJavaObject::from_i16(env, val as i16)?.into()
                }
            }
            Type::String | Type::CharSequence => {
                if value.get_type()? == ValueType::Object {
                    JavaObject::from(value.into_java_object(node_env)?)
                } else {
                    let val = value.coerce_to_string()?.into_utf16()?.as_str()?;
                    JavaString::from_string(val, env)?.into()
                }
            }
            Type::Object | Type::LangObject => match value.get_type()? {
                ValueType::Null | ValueType::Undefined => return Ok(None),
                ValueType::Boolean => {
                    LocalJavaObject::from_bool(env, value.coerce_to_bool()?.get_value()?)?.into()
                }
                ValueType::String => {
                    let val = value.coerce_to_string()?.into_utf16()?.as_str()?;
                    JavaString::from_string(val, env)?.into()
                }
                ValueType::Number => {
                    let number = value.coerce_to_number()?;
                    if is_integer(node_env, &number)? {
                        let val = number.get_int32()?;
                        LocalJavaObject::from_i32(env, val)?.into()
                    } else {
                        let val = number.get_double()?;
                        LocalJavaObject::from_f64(env, val)?.into()
                    }
                }
                ValueType::BigInt => {
                    let val = unsafe { value.cast::<JsBigInt>() }.get_i64()?.0;
                    LocalJavaObject::from_i64(env, val)?.into()
                }
                ValueType::Object => {
                    let err_fn = |_| "Expected a java object as parameter".to_string();

                    if value.is_array()? {
                        let arr = unsafe { value.cast::<JsTypedArray>() };
                        let class = JavaClass::by_name("java/lang/Object", env)?;
                        let mut res = JavaObjectArray::new(&class, arr.get_array_length()? as _)?;
                        for i in 0..res.len()? {
                            res.set(
                                i,
                                self.convert_to_java_object(
                                    env,
                                    node_env,
                                    arr.get_element(i as _)?,
                                )?,
                            )?;
                        }

                        return Ok(Some(GlobalJavaObject::try_from(res.into_object())?.into()));
                    }

                    let obj = value.coerce_to_object().map_err(err_fn)?;
                    if JavaInterfaceProxy::instance_of(*node_env, &obj)? {
                        let proxy: JsUnknown = obj.get_named_property("proxy")?;
                        if proxy.get_type()? == ValueType::Null {
                            return Err("The proxy has already been destroyed".into());
                        }

                        JavaObject::from(
                            node_env
                                .unwrap::<GlobalJavaObject>(&proxy.coerce_to_object()?)?
                                .clone(),
                        )
                    } else {
                        let js_obj: JsObject =
                            obj.get_named_property(OBJECT_PROPERTY).map_err(err_fn)?;
                        let java_obj: &mut GlobalJavaObject =
                            node_env.unwrap(&js_obj).map_err(err_fn)?;

                        let signature = java_obj.get_signature();
                        let class = JavaClass::by_java_name(self.to_string(), env)?;

                        if !class.is_assignable_from(&java_obj.get_class(env)?)? {
                            return Err(
                                format!("{} is not assignable to {}", signature, self).into()
                            );
                        }

                        java_obj.clone().into()
                    }
                }
                _ => {
                    return Err("Invalid value type supplied".to_string().into());
                }
            },
            Type::Array => self.array_to_java_value(env, node_env, value)?,
            Type::Void => {
                return Err("Cannot use 'void' as input type".into());
            }
        }))
    }

    fn convert_to_java_value<'a>(
        &self,
        env: &'a JavaEnv<'a>,
        node_env: &'a Env,
        value: JsUnknown,
    ) -> ResultType<JavaCallResult> {
        Ok(if value.get_type()? == ValueType::Object {
            match self.type_enum() {
                Type::Integer => JavaCallResult::Integer(env.object_to_int(
                    &LocalJavaObject::from(&value.into_java_object(node_env)?, env),
                )?),
                Type::Long => JavaCallResult::Long(env.object_to_long(&LocalJavaObject::from(
                    &value.into_java_object(node_env)?,
                    env,
                ))?),
                Type::Float => JavaCallResult::Float(env.object_to_float(
                    &LocalJavaObject::from(&value.into_java_object(node_env)?, env),
                )?),
                Type::Double => JavaCallResult::Double(env.object_to_double(
                    &LocalJavaObject::from(&value.into_java_object(node_env)?, env),
                )?),
                Type::Boolean => JavaCallResult::Boolean(env.object_to_boolean(
                    &LocalJavaObject::from(&value.into_java_object(node_env)?, env),
                )?),
                Type::Byte => JavaCallResult::Byte(env.object_to_byte(&LocalJavaObject::from(
                    &value.into_java_object(node_env)?,
                    env,
                ))?),
                Type::Short => JavaCallResult::Short(env.object_to_short(
                    &LocalJavaObject::from(&value.into_java_object(node_env)?, env),
                )?),
                Type::Character => JavaCallResult::Character(env.object_to_char(
                    &LocalJavaObject::from(&value.into_java_object(node_env)?, env),
                )?),
                _ => match self.convert_to_java_object(env, node_env, value)? {
                    Some(obj) => JavaCallResult::Object {
                        object: obj.try_into()?,
                        signature: self.clone(),
                    },
                    None => JavaCallResult::Null,
                },
            }
        } else {
            match self.type_enum() {
                Type::Integer => JavaCallResult::Integer(value.coerce_to_number()?.get_int32()?),
                Type::Long => {
                    if value.get_type()? == ValueType::BigInt {
                        JavaCallResult::Long(unsafe { value.cast::<JsBigInt>() }.get_i64()?.0)
                    } else {
                        JavaCallResult::Long(value.coerce_to_number()?.get_int64()?)
                    }
                }
                Type::Short => JavaCallResult::Short(value.coerce_to_number()?.get_int32()? as i16),
                Type::Double => JavaCallResult::Double(value.coerce_to_number()?.get_double()?),
                Type::Float => {
                    JavaCallResult::Float(value.coerce_to_number()?.get_double()? as f32)
                }
                Type::Byte => JavaCallResult::Byte(value.coerce_to_number()?.get_int32()? as i8),
                Type::Character => {
                    let str = value.coerce_to_string()?.into_utf16()?;
                    let arr = str.as_slice();
                    if arr.len() == 1 {
                        JavaCallResult::Character(arr[0])
                    } else {
                        return Err("Java character must be a single character".into());
                    }
                }
                Type::Boolean => JavaCallResult::Boolean(value.coerce_to_bool()?.get_value()?),
                _ => {
                    if value.get_type()? == ValueType::Null
                        || value.get_type()? == ValueType::Undefined
                    {
                        JavaCallResult::Null
                    } else if let Some(obj) = self.convert_to_java_object(env, node_env, value)? {
                        JavaCallResult::Object {
                            object: obj.try_into()?,
                            signature: self.clone(),
                        }
                    } else {
                        JavaCallResult::Null
                    }
                }
            }
        })
    }

    fn array_to_java_value<'a>(
        &self,
        env: &'a JavaEnv<'a>,
        node_env: &'a Env,
        value: JsUnknown,
    ) -> ResultType<JavaObject<'a>> {
        if !self.is_array() {
            return Err("Type must be an array".into());
        } else if value.is_buffer()? && self.is_byte_array() {
            let buffer = unsafe { value.cast::<JsBuffer>() }.into_value()?;

            let mut vec: Vec<i8> = Vec::with_capacity(buffer.len());
            for i in 0..buffer.len() {
                vec.push(
                    *buffer
                        .get(i)
                        .ok_or(format!("Failed to get buffer element at position {}", i))?
                        as i8,
                );
            }

            return Ok(JavaByteArray::new(env, &vec)?.into());
        } else if !value.is_array()? {
            return Err("Value must be an array".into());
        }

        let array = unsafe { value.cast::<JsTypedArray>() };
        let length = array.get_array_length()?;
        let inner = self
            .inner()
            .ok_or("Array value has no inner type")?
            .lock()
            .unwrap()
            .clone();

        Ok(match inner.type_enum() {
            Type::Integer => {
                let mut res: Vec<i32> = vec![];
                for i in 0..length {
                    res.push(array.get_element::<JsNumber>(i)?.get_int32()?);
                }

                JavaIntArray::new(env, &res)?.into()
            }
            Type::Long => {
                let mut res: Vec<i64> = vec![];
                for i in 0..length {
                    res.push(array.get_element::<JsNumber>(i)?.get_int64()?);
                }

                JavaLongArray::new(env, &res)?.into()
            }
            Type::Float => {
                let mut res: Vec<f32> = vec![];
                for i in 0..length {
                    res.push(array.get_element::<JsNumber>(i)?.get_double()? as f32);
                }

                JavaFloatArray::new(env, &res)?.into()
            }
            Type::Double => {
                let mut res: Vec<f64> = vec![];
                for i in 0..length {
                    res.push(array.get_element::<JsNumber>(i)?.get_double()?);
                }

                JavaDoubleArray::new(env, &res)?.into()
            }
            Type::Byte => {
                let mut res: Vec<i8> = vec![];
                for i in 0..length {
                    res.push(array.get_element::<JsNumber>(i)?.get_int32()? as i8);
                }

                JavaByteArray::new(env, &res)?.into()
            }
            Type::Short => {
                let mut res: Vec<i16> = vec![];
                for i in 0..length {
                    res.push(array.get_element::<JsNumber>(i)?.get_int32()? as i16);
                }

                JavaShortArray::new(env, &res)?.into()
            }
            Type::Character => {
                let mut res = Vec::new();
                for i in 0..length {
                    let str = array.get_element::<JsString>(i)?.into_utf16()?;
                    let slice = str.as_slice();
                    if slice.len() == 2 && slice[1] == 0 {
                        res.push(slice[0]);
                    } else {
                        return Err("Java character must be a single character".into());
                    }
                }

                JavaCharArray::new(env, &res)?.into()
            }
            Type::Boolean => {
                let mut res: Vec<u8> = vec![];
                for i in 0..length {
                    res.push(array.get_element::<JsBoolean>(i)?.get_value()?.into());
                }

                JavaBooleanArray::new(env, &res)?.into()
            }
            _ => {
                let class = JavaClass::by_java_name(
                    self.get_most_inner_signature().replace('/', "."),
                    env,
                )?;
                let mut res = JavaObjectArray::new(&class, length as usize)?;
                for i in 0..length {
                    res.set(
                        i as i32,
                        inner.convert_to_java_object(env, node_env, array.get_element(i)?)?,
                    )?;
                }

                GlobalJavaObject::try_from(res.into_object())?.into()
            }
        })
    }
}
