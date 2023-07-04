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
    fn map_napi_err(self) -> napi::Result<T>;
}

impl<T, E> MapToNapiError<T> for Result<T, E>
where
    E: ToString,
{
    fn map_napi_err(self) -> napi::Result<T> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(NapiError::from(err.to_string()).into_napi()),
        }
    }
}

pub trait StrIntoNapiError {
    fn into_napi_err(self) -> napi::Error;
}

impl StrIntoNapiError for &str {
    fn into_napi_err(self) -> napi::Error {
        NapiError::from(self).into()
    }
}
