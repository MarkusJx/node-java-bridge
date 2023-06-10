use napi::JsUnknown;
use std::sync::Once;

fn warn_disabled() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        eprintln!("WARNING: logging is not supported in this build");
    });
}

#[napi]
/// This method is not supported in this build.
/// It will print a warning to stderr when called.
///
/// Re-compile the native module with the `log` feature to enable logging.
pub fn set_log_callbacks_internal(
    #[napi(ts_arg_type = "((err: object | null, data: string | null) => void) | null")]
    _out: JsUnknown,
    #[napi(ts_arg_type = "((err: object | null, data: string | null) => void) | null")]
    _err: JsUnknown,
) {
    warn_disabled()
}

#[napi]
/// This method is not supported in this build.
/// It will print a warning to stderr when called.
///
/// Re-compile the native module with the `log` feature to enable logging.
pub fn init_logger(_path: String) {
    warn_disabled()
}

#[napi]
/// This method is not supported in this build.
/// It will print a warning to stderr when called.
///
/// Re-compile the native module with the `log` feature to enable logging.
pub fn reset_log_callbacks() {
    warn_disabled()
}
