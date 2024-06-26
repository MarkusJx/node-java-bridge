#[napi]
/// A namespace containing logging functions.
///
/// All methods in this namespace are dummies.
/// They will print a warning once to stderr when called.
///
/// Re-compile the native module with the `log` feature to enable logging.
pub mod logging {
    use napi::JsUnknown;
    use std::sync::Once;

    #[allow(unused)]
    fn warn_disabled() {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            eprintln!("WARNING: logging is not supported in this build");
        });
    }

    #[napi]
    #[allow(unused)]
    /// This method is not supported in this build.
    /// It will print a warning to stderr when called.
    ///
    /// Re-compile the native module with the `log` feature to enable logging.
    pub fn set_log_callbacks(
        #[napi(ts_arg_type = "((data: string | null) => void) | null | undefined")] _out: JsUnknown,
        #[napi(ts_arg_type = "((data: string | null) => void) | null | undefined")] _err: JsUnknown,
    ) {
        warn_disabled()
    }

    #[napi]
    #[allow(unused)]
    /// This method is not supported in this build.
    /// It will print a warning to stderr when called.
    ///
    /// Re-compile the native module with the `log` feature to enable logging.
    pub fn init_logger(_path: String) {
        warn_disabled()
    }

    #[napi]
    #[allow(unused)]
    /// This method is not supported in this build.
    /// It will print a warning to stderr when called.
    ///
    /// Re-compile the native module with the `log` feature to enable logging.
    pub fn reset_log_callbacks() {
        warn_disabled()
    }

    #[napi]
    #[allow(unused)]
    /// Whether logging is supported.
    /// Logging is disabled by default.
    /// This constant currently is set to `false`
    /// as logging is not supported in this build.
    pub const LOGGING_SUPPORTED: bool = false;
}
