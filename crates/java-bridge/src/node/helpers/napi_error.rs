use crate::debug;
use crate::node::class_cache::ClassCache;
use crate::node::java_class_instance::JavaClassInstance;
use crate::node::util::util::ResultType;
use app_state::{AppStateTrait, MutAppState};
use java_rs::java_error::JavaError;
use java_rs::objects::java_object::JavaObject;
use napi::{JsError, Property, PropertyAttributes};
use std::error::Error;

pub struct NapiError(napi::Error);

impl NapiError {
    fn new(status: napi::Status, msg: String) -> Self {
        NapiError(napi::Error::new(status, msg))
    }

    pub fn into_napi(self) -> napi::Error {
        self.0
    }
}

impl From<Box<dyn Error>> for NapiError {
    fn from(err: Box<dyn Error>) -> Self {
        NapiError::new(napi::Status::GenericFailure, err.to_string())
    }
}

impl From<String> for NapiError {
    fn from(msg: String) -> Self {
        NapiError::new(napi::Status::GenericFailure, msg)
    }
}

impl From<&str> for NapiError {
    fn from(msg: &str) -> Self {
        NapiError::new(napi::Status::GenericFailure, msg.to_string())
    }
}

impl From<Option<Box<dyn Error>>> for NapiError {
    fn from(err: Option<Box<dyn Error>>) -> Self {
        match err {
            Some(err) => NapiError::new(napi::Status::GenericFailure, err.to_string()),
            None => NapiError::new(napi::Status::GenericFailure, "Unknown error".to_string()),
        }
    }
}

impl Into<napi::Error> for NapiError {
    fn into(self) -> napi::Error {
        self.0
    }
}

pub trait MapToNapiError<T> {
    fn map_napi_err(self, env: Option<napi::Env>) -> napi::Result<T>;
}

impl<T, E> MapToNapiError<T> for Result<T, E>
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    fn map_napi_err(self, env: Option<napi::Env>) -> napi::Result<T> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => {
                let error_box = err.into();
                let napi_error = NapiError::from(error_box.to_string()).into_napi();

                match convert_error(env, &error_box, &napi_error) {
                    Ok(napi_error) => Err(napi_error),
                    Err(_err) => {
                        debug!("Failed to convert error: {_err}");
                        Err(napi_error)
                    }
                }
            }
        }
    }
}

fn convert_error(
    env: Option<napi::Env>,
    error: &Box<dyn Error + Send + Sync>,
    napi_error: &napi::Error,
) -> ResultType<napi::Error> {
    let env = env.ok_or("Missing n-api environment")?;
    let java_err = error
        .downcast_ref::<JavaError>()
        .ok_or("Error is not of type 'JavaError'")?;
    let throwable = java_err.get_throwable().ok_or("Missing java throwable")?;
    let mut error_obj = JsError::from(napi_error.clone())
        .into_unknown(env)
        .coerce_to_object()?;

    let vm = throwable.get_vm();
    let java_env = vm.attach_thread()?;
    let throwable_name = java_env.get_class_name(JavaObject::from(&throwable))?;

    let proxy = MutAppState::<ClassCache>::get_or_insert_default()
        .try_lock()
        .map_err(|_| "Failed to lock class cache")?
        .get_class_proxy(&vm, throwable_name, None)?;
    let converted_throwable = JavaClassInstance::from_existing(proxy, &env, throwable)?;
    error_obj.define_properties(&[Property::new("cause")?
        .with_value(&converted_throwable)
        .with_property_attributes(
            PropertyAttributes::Enumerable | PropertyAttributes::Configurable,
        )])?;

    Ok(napi::Error::from(error_obj.into_unknown()))
}

pub trait StrIntoNapiError {
    fn into_napi_err(self) -> napi::Error;
}

impl StrIntoNapiError for &str {
    fn into_napi_err(self) -> napi::Error {
        NapiError::from(self).into()
    }
}
