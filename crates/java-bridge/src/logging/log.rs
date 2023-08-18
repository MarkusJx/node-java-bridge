#[napi]
/// A namespace containing logging functions.
///
/// The methods in this namespace are only available
/// if the native library was compiled with the `log` feature.
/// By default, the native library is compiled without this feature.
///
/// If you don't know if the native library was compiled with the `log` feature,
/// the jsdoc comments of all methods in this namespace will start with
/// "This method is not supported in this build" if the native library
/// has been compiled without the `log` feature. Otherwise, the usual
/// jsdoc comments will be present. Also, the {@link internal.logging.LOGGING_SUPPORTED}
/// constant can be used to check if the native library was compiled with
/// the `log` feature.
///
/// ## Example
/// ```ts
/// import { logging } from 'java-bridge';
///
/// logging.initLogger('log4rs.json');
/// logging.setLogCallbacks(
///   (out) => console.log(out),
///   (err) => console.error(err)
/// );
/// ```
///
/// See {@link logging.initLogger} for further information
/// on how to configure the logger.
///
/// @since 2.4.0
mod logging {
    use crate::logging::appender::NodeAppenderSerializer;
    use crate::logging::writer::NodeWriter;
    use crate::node::helpers::napi_error::MapToNapiError;
    use lazy_static::lazy_static;
    use log::{debug, info};
    use log4rs::config::{load_config_file, Deserializers};
    use log4rs::Handle;
    use napi::threadsafe_function::ErrorStrategy;
    use napi::threadsafe_function::ThreadsafeFunction;
    use std::sync::Mutex;

    lazy_static! {
        static ref HANDLE: Mutex<Option<Handle>> = Mutex::new(None);
    }

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
    /// The log callbacks can be set using the {@link logging.setLogCallbacks} function.
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

        let config = load_config_file(path, deserializers).map_napi_err()?;
        let mut handle_guard = HANDLE.lock().map_napi_err()?;
        if let Some(handle) = handle_guard.as_ref() {
            info!("Logger already initialized, updating configuration");
            handle.set_config(config);
        } else {
            let handle = log4rs::init_config(config).map_napi_err()?;
            handle_guard.replace(handle);
        }

        info!("Logger initialized");
        Ok(())
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
    /// **Note:** The callbacks must not throw any exceptions.
    /// If an exception is thrown, the process will exit.
    ///
    /// ## Example
    /// ```ts
    /// import { logging } from 'java-bridge';
    ///
    /// logging.setLogCallbacks(
    ///   (out) => console.log(out),
    ///   (err) => console.error(err)
    /// );
    /// ```
    ///
    /// @param out The callback for the stdout log level.
    /// @param err The callback for the stderr log level.
    pub fn set_log_callbacks(
        #[napi(ts_arg_type = "((data: string) => void) | null | undefined")] out: Option<
            ThreadsafeFunction<String, ErrorStrategy::Fatal>,
        >,
        #[napi(ts_arg_type = "((data: string) => void) | null | undefined")] err: Option<
            ThreadsafeFunction<String, ErrorStrategy::Fatal>,
        >,
    ) -> napi::Result<()> {
        NodeWriter::set_callbacks(out, err);
        debug!("Log callbacks set");
        Ok(())
    }

    #[napi]
    /// Reset the log callbacks for the `node` log appender.
    /// This will disable the `node` appender.
    /// The default log4rs appenders will still be available.
    /// Call {@link logging.setLogCallbacks} to enable the `node` appender again.
    pub fn reset_log_callbacks() -> napi::Result<()> {
        debug!("Resetting log callbacks");
        Ok(NodeWriter::set_callbacks(None, None))
    }

    #[napi]
    /// Whether logging is supported.
    /// Logging is disabled by default.
    /// This constant is currently set to `true`
    /// as logging is supported in this build.
    pub const LOGGING_SUPPORTED: bool = true;
}
