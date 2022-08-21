use crate::jni::java_call_result::JavaCallResult;
use crate::jni::java_env::JavaEnv;
use crate::jni::java_type::{JavaType, Type};
use crate::jni::objects::args::JavaArg;
use crate::jni::objects::array::{
    JavaBooleanArray, JavaByteArray, JavaCharArray, JavaDoubleArray, JavaFloatArray, JavaIntArray,
    JavaLongArray, JavaObjectArray, JavaShortArray,
};
use crate::jni::objects::class::JavaClass;
use crate::jni::objects::java_object::JavaObject;
use crate::jni::objects::object::{GlobalJavaObject, LocalJavaObject};
use crate::jni::objects::string::JavaString;
use crate::jni::traits::GetSignature;
use crate::jni::util::util::ResultType;
use crate::node::java_class_instance::OBJECT_PROPERTY;
use crate::node::java_interface_proxy::JavaInterfaceProxy;
use napi::{
    Env, JsBigInt, JsBoolean, JsFunction, JsNumber, JsObject, JsString, JsTypedArray, JsUnknown,
    ValueType,
};
use std::ops::Deref;

fn value_may_be_byte(value: &JsUnknown) -> bool {
    let val = unsafe { value.cast::<JsNumber>() }
        .get_double()
        .unwrap_or(-1.0);
    val >= -128.0 && val <= 127.0 && val.round() == val
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

impl PartialEq<JsUnknown> for JavaType {
    fn eq(&self, other: &JsUnknown) -> bool {
        match other.get_type().unwrap() {
            ValueType::String => {
                Type::String == self
                    || (unsafe { other.cast::<JsString>() }.utf16_len().unwrap() == 1
                        && self.is_char())
            }
            ValueType::Number => {
                self.is_int()
                    || self.is_double()
                    || self.is_float()
                    || self.is_long()
                    || (value_may_be_byte(other) && self.is_byte())
            }
            ValueType::Boolean => self.is_boolean(),
            ValueType::BigInt => self.is_long(),
            _ => {
                if other.is_array().unwrap() && self.is_array() {
                    let arr = unsafe { other.cast::<JsTypedArray>() };
                    if arr.get_array_length().unwrap() == 0 {
                        true
                    } else {
                        self.inner().unwrap().lock().unwrap().deref()
                            == &arr.get_element::<JsUnknown>(0).unwrap()
                    }
                } else if other.is_buffer().unwrap() {
                    self.is_byte_array()
                } else {
                    Type::Object == self
                }
            }
        }
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
    ) -> ResultType<JavaObject<'a>>;

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
    ) -> ResultType<JavaObject<'a>> {
        Ok(match self.type_enum() {
            Type::LangInteger | Type::Integer => {
                let val = value.coerce_to_number()?.get_int32()?;
                JavaObject::from(LocalJavaObject::from_i32(env, val)?)
            }
            Type::LangLong | Type::Long => {
                let val = value.coerce_to_number()?.get_int64()?;
                LocalJavaObject::from_i64(env, val)?.into()
            }
            Type::LangFloat | Type::Float => {
                let val = value.coerce_to_number()?.get_double()?;
                LocalJavaObject::from_f32(env, val as f32)?.into()
            }
            Type::LangDouble | Type::Double => {
                let val = value.coerce_to_number()?.get_double()?;
                LocalJavaObject::from_f64(env, val)?.into()
            }
            Type::LangBoolean | Type::Boolean => {
                let val = value.coerce_to_bool()?.get_value()?;
                LocalJavaObject::from_bool(env, val)?.into()
            }
            Type::LangCharacter | Type::Character => {
                let val = value.coerce_to_string()?.into_utf16()?;
                let slice = val.as_slice();

                if slice.len() == 1 {
                    LocalJavaObject::from_char(env, slice[0])?.into()
                } else {
                    return Err("Java character must be a single character".into());
                }
            }
            Type::LangByte | Type::Byte => {
                let val = value.coerce_to_number()?.get_int32()?;
                LocalJavaObject::from_byte(env, val as i8)?.into()
            }
            Type::LangShort | Type::Short => {
                let val = value.coerce_to_number()?.get_int32()?;
                LocalJavaObject::from_i16(env, val as i16)?.into()
            }
            Type::String => {
                let val = value.coerce_to_string()?.into_utf16()?.as_str()?;
                JavaString::try_from(val, env)?.into()
            }
            Type::Object | Type::LangObject => match value.get_type()? {
                ValueType::Null | ValueType::Undefined => GlobalJavaObject::null(env)?.into(),
                ValueType::Boolean => {
                    LocalJavaObject::from_bool(env, value.coerce_to_bool()?.get_value()?)?.into()
                }
                ValueType::String => {
                    let val = value.coerce_to_string()?.into_utf16()?.as_str()?;
                    JavaString::try_from(val, env)?.into()
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

                        return Ok(GlobalJavaObject::try_from(res.into_object())?.into());
                    }

                    let obj = value.coerce_to_object().map_err(err_fn)?;
                    if JavaInterfaceProxy::instance_of(node_env.clone(), &obj)? {
                        let proxy: JsObject = obj.get_named_property("proxy")?;
                        JavaObject::from(node_env.unwrap::<GlobalJavaObject>(&proxy)?.clone())
                    } else {
                        let js_obj: JsObject =
                            obj.get_named_property(OBJECT_PROPERTY).map_err(err_fn)?;
                        let java_obj: &mut GlobalJavaObject =
                            node_env.unwrap(&js_obj).map_err(err_fn)?;

                        let signature = java_obj.get_signature()?;
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
        })
    }

    fn convert_to_java_value<'a>(
        &self,
        env: &'a JavaEnv<'a>,
        node_env: &'a Env,
        value: JsUnknown,
    ) -> ResultType<JavaCallResult> {
        Ok(match self.type_enum() {
            Type::Integer => JavaCallResult::Integer(value.coerce_to_number()?.get_int32()?),
            Type::Long => JavaCallResult::Long(value.coerce_to_number()?.get_int64()?),
            Type::Short => JavaCallResult::Short(value.coerce_to_number()?.get_int32()? as i16),
            Type::Double => JavaCallResult::Double(value.coerce_to_number()?.get_double()?),
            Type::Float => JavaCallResult::Float(value.coerce_to_number()?.get_double()? as f32),
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
                if value.get_type()? == ValueType::Null || value.get_type()? == ValueType::Undefined
                {
                    JavaCallResult::Null
                } else {
                    JavaCallResult::Object {
                        object: self
                            .convert_to_java_object(env, node_env, value)?
                            .try_into()?,
                        signature: self.clone(),
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
        if !value.is_array()? {
            return Err("Value must be an array".into());
        } else if !self.is_array() {
            return Err("Type must be an array".into());
        }

        let inner = self
            .inner()
            .ok_or("Array value has no inner type")?
            .lock()
            .unwrap()
            .clone();

        let array = unsafe { value.cast::<JsTypedArray>() };
        let length = array.get_array_length()?;

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
                let class = JavaClass::by_name(self.get_most_inner_signature().as_str(), env)?;
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
