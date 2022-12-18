use napi::{sys, CallContext, Env, NapiRaw, Status};
use std::os::raw::c_void;
use std::panic::AssertUnwindSafe;
use std::{panic, ptr};

pub(crate) fn get_trampoline_args(
    raw_env: sys::napi_env,
    cb_info: sys::napi_callback_info,
) -> (
    *mut sys::napi_value__,
    Vec<*mut sys::napi_value__>,
    *mut c_void,
) {
    let argc = {
        let mut argc = 0;
        let status = unsafe {
            sys::napi_get_cb_info(
                raw_env,
                cb_info,
                &mut argc,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        debug_assert!(
            Status::from(status) == Status::Ok,
            "napi_get_cb_info failed"
        );
        argc
    };
    let mut raw_args = vec![ptr::null_mut(); argc];
    let mut raw_this = ptr::null_mut();
    let mut closure_data_ptr = ptr::null_mut();

    let status = unsafe {
        sys::napi_get_cb_info(
            raw_env,
            cb_info,
            &mut { argc },
            raw_args.as_mut_ptr(),
            &mut raw_this,
            &mut closure_data_ptr,
        )
    };
    debug_assert!(
        Status::from(status) == Status::Ok,
        "napi_get_cb_info failed"
    );
    (raw_this, raw_args, closure_data_ptr)
}

pub(crate) fn call_trampoline_func<
    F: Fn(CallContext, *mut c_void) -> napi::Result<R>,
    R: NapiRaw,
>(
    raw_env: sys::napi_env,
    cb_info: sys::napi_callback_info,
    cb: F,
) -> sys::napi_value {
    panic::catch_unwind(AssertUnwindSafe(|| {
        let (raw_this, ref raw_args, data_ptr) = get_trampoline_args(raw_env, cb_info);

        let env = &mut unsafe { Env::from_raw(raw_env) };
        let ctx = CallContext::new(env, cb_info, raw_this, raw_args, raw_args.len());

        cb(ctx, data_ptr).map(|v| unsafe { v.raw() })
    }))
    .map_err(|e| {
        napi::Error::from_reason(format!(
            "panic from Rust code: {}",
            if let Some(s) = e.downcast_ref::<String>() {
                s
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s
            } else {
                "<no error message>"
            },
        ))
    })
    .and_then(|v| v)
    .unwrap_or_else(|e| {
        unsafe { napi::JsError::from(e).throw_into(raw_env) };
        ptr::null_mut()
    })
}
