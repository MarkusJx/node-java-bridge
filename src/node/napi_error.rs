use crate::jni::util::util::ResultType;
use std::error::Error;

pub struct NapiError(napi::Error);

impl NapiError {
    fn new(status: napi::Status, msg: String) -> Self {
        NapiError(napi::Error::new(status, msg))
    }

    pub fn to_napi_error(err: Box<dyn Error>) -> napi::Error {
        NapiError::from(err).into()
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
    fn map_napi_err(self) -> napi::Result<T>;
}

impl<T> MapToNapiError<T> for ResultType<T> {
    fn map_napi_err(self) -> napi::Result<T> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(NapiError::from(err).into()),
        }
    }
}
