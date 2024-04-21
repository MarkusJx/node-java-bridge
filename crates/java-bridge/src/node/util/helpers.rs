use crate::node::extensions::java_call_result_ext::ToNapiValue;
use crate::node::helpers::napi_error::{MapToNapiError, StrIntoNapiError};
use crate::node::java_class_proxy::JavaClassProxy;
use glob::glob;
use java_rs::java_call_result::JavaCallResult;
use napi::{Env, JsObject, JsString, JsUnknown, NapiRaw};
use std::error::Error;
use std::sync::Arc;

pub type ResultType<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[cfg(windows)]
mod separator {
    pub const CLASSPATH_SEPARATOR: &str = ";";
    pub const OTHER_SEPARATOR: &str = ":";
}
#[cfg(unix)]
mod separator {
    pub const CLASSPATH_SEPARATOR: &str = ":";
    pub const OTHER_SEPARATOR: &str = ";";
}

/// Parse an JsUnknown that is either a JsString or a JsArray into a String
pub(crate) fn parse_array_or_string(value: JsUnknown) -> napi::Result<Vec<String>> {
    let mut res = Vec::<String>::new();
    if value.is_array()? {
        let obj = value.coerce_to_object()?;
        for i in 0..obj.get_array_length()? {
            let path: JsString = obj.get_element(i)?;
            res.push(path.into_utf16()?.as_str()?);
        }
    } else {
        let path = value.coerce_to_string()?;
        res.push(path.into_utf16()?.as_str()?);
    }

    Ok(res)
}

pub(crate) fn list_files(dirs: Vec<String>, ignore_unreadable: bool) -> napi::Result<Vec<String>> {
    dirs.into_iter()
        .map(|f| glob(f.as_str()).map_napi_err(None))
        .collect::<napi::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .map(|f| f.map_napi_err(None))
        .filter_map(|f| match f {
            Ok(f) => Some(
                f.to_str()
                    .ok_or("Failed to convert path to string".into_napi_err())
                    .map(|f| f.to_string()),
            ),
            Err(e) => {
                if ignore_unreadable {
                    None
                } else {
                    Some(Err(e))
                }
            }
        })
        .collect()
}

pub fn parse_classpath_args(cp: &[String], args: &mut Vec<String>) -> String {
    let mut cp = cp.to_vec();
    if let Some(other) = args
        .iter_mut()
        .position(|e| e.starts_with("-Djava.class.path="))
    {
        let other_cp = args.remove(other).split_at(18).1.to_string();
        cp.push(other_cp.replace(separator::OTHER_SEPARATOR, separator::CLASSPATH_SEPARATOR));
    }

    format!(
        "-Djava.class.path={}",
        cp.join(separator::CLASSPATH_SEPARATOR)
    )
}

pub fn call_async_method<F>(env: Env, proxy: Arc<JavaClassProxy>, func: F) -> napi::Result<JsObject>
where
    F: (FnOnce() -> ResultType<JavaCallResult>) + Send + Sync + 'static,
{
    call_async_method_with_resolver(
        env,
        proxy.async_java_exception_objects(),
        func,
        move |&mut env, res| {
            let j_env = proxy.vm.attach_thread().map_napi_err(Some(env))?;
            res.to_napi_value(&j_env, &env).map_napi_err(Some(env))
        },
    )
}

pub fn call_async_method_with_resolver<F, Res, R, V>(
    env: Env,
    async_java_exception_objects: bool,
    func: F,
    resolver: Res,
) -> napi::Result<JsObject>
where
    F: (FnOnce() -> ResultType<R>) + Send + Sync + 'static,
    Res: FnOnce(&mut Env, R) -> napi::Result<V> + Send + Sync + 'static,
    R: Send + Sync + 'static,
    V: NapiRaw + 'static,
{
    if async_java_exception_objects {
        env.execute_tokio_future(futures::future::lazy(|_| Ok(func())), move |env, res| {
            resolver(env, res.map_napi_err(Some(*env))?)
        })
    } else {
        env.execute_tokio_future(
            futures::future::lazy(move |_| func().map_napi_err(None)),
            resolver,
        )
    }
}
