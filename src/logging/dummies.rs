use napi::JsUnknown;
use std::sync::Once;

fn warn_disabled() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        eprintln!("WARNING: logging is not supported in this build");
    });
}

#[napi]
pub fn set_log_callbacks(
    #[napi(ts_arg_type = "((err: object | null, data: string | null) => void) | null")]
    _out: JsUnknown,
    #[napi(ts_arg_type = "((err: object | null, data: string | null) => void) | null")]
    _err: JsUnknown,
) {
    warn_disabled()
}

#[napi]
pub fn init_logger(_path: String) {
    warn_disabled()
}

#[napi]
pub fn reset_log_callbacks() {
    warn_disabled()
}
