use crate::logging::appender::NodeAppenderSerializer;
use crate::logging::writer::NodeWriter;
use crate::node::helpers::napi_error::MapToNapiError;
use log::{debug, info};
use log4rs::config::Deserializers;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::{Env, JsFunction};

#[napi]
pub fn init_logger(path: String) -> napi::Result<()> {
    let mut deserializers = Deserializers::default();
    deserializers.insert("node", NodeAppenderSerializer);

    log4rs::init_file(path, deserializers).map_napi_err()?;

    info!("Logger initialized");
    Ok(())
}

fn create_threadsafe_fn(
    env: &Env,
    func: Option<JsFunction>,
) -> napi::Result<Option<ThreadsafeFunction<String>>> {
    func.as_ref()
        .map(|out| {
            env.create_threadsafe_function(out, 0, |ctx| {
                Ok(vec![ctx.env.create_string_from_std(ctx.value)?])
            })
        })
        .map_or(Ok(None), |r| r.map(Some).map_napi_err())
}

#[napi]
pub fn set_log_callbacks(
    env: Env,
    #[napi(ts_arg_type = "((err?: object | null, data?: string | null) => void) | null")]
    out: Option<JsFunction>,
    #[napi(ts_arg_type = "((err?: object | null, data?: string | null) => void) | null")]
    err: Option<JsFunction>,
) -> napi::Result<()> {
    if out.is_none() && err.is_none() {
        debug!("Resetting log callbacks");
    } else {
        debug!("Setting log callbacks");
    }

    let out = create_threadsafe_fn(&env, out)?;
    let err = create_threadsafe_fn(&env, err)?;

    NodeWriter::set_callbacks(out, err);
    debug!("Log callbacks set");
    Ok(())
}

#[napi]
pub fn reset_log_callbacks() -> napi::Result<()> {
    debug!("Resetting log callbacks");
    Ok(NodeWriter::set_callbacks(None, None))
}
