use log::{debug, info};
use log4rs::config::Deserializers;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::{Env, JsFunction};

use crate::logging::appender::NodeAppenderSerializer;
use crate::logging::writer::NodeWriter;
use crate::node::helpers::napi_error::MapToNapiError;

#[napi]
/// Initializes the logger with the given configuration file.
/// The configuration file must be a valid [log4rs](https://docs.rs/log4rs/latest/log4rs/)
/// configuration file. Accepted formats are yaml and json.
///
/// In addition to the default log4rs appenders, a custom
/// appender called `node` is available. This appender
/// will write to provided Node.js callbacks.
///
/// The `node` appender accepts the default log4rs encoder argument.
/// The log callbacks can be set using the {@link setLogCallbacks} function.
///
/// ## Example configuration (json)
/// ```json
/// {
///     "appenders": {
///         "stdout": {
///             "kind": "console"
///         },
///         "node": {
///             "kind": "node",
///             "encoder": {
///                 "pattern": "{d} [{t}] {m}{n}"
///             }
///         },
///         "file": {
///             "kind": "file",
///             "path": "app.log",
///             "append": false
///         }
///     },
///     "root": {
///         "level": "trace",
///         "appenders": ["node", "file"]
///     }
///  }
/// ```
///
/// @param path The path to the configuration file.
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
/// Set the log callbacks for the `node` log appender.
/// In order to enable the `node` appender, the callbacks must be set.
/// By default info, debug and trace messages are written to stdout.
/// Error, warn and fatal messages are written to stderr.
/// If one of the callbacks is not set, all messages will be written
/// to the other callback. If both callbacks are not set, the `node`
/// appender will be disabled.
///
/// @param out The callback for the stdout log level.
/// @param err The callback for the stderr log level.
pub fn set_log_callbacks_internal(
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
/// Reset the log callbacks for the `node` log appender.
/// This will disable the `node` appender.
/// The default log4rs appenders will still be available.
/// Call {@link setLogCallbacks} to enable the `node` appender again.
pub fn reset_log_callbacks() -> napi::Result<()> {
    debug!("Resetting log callbacks");
    Ok(NodeWriter::set_callbacks(None, None))
}
