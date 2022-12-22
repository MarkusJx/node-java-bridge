use crate::node::java_call_result_ext::ToNapiValue;
use crate::node::java_class_instance::{CLASS_PROXY_PROPERTY, OBJECT_PROPERTY};
use crate::node::java_class_proxy::JavaClassProxy;
use crate::node::java_type_ext::NapiToJava;
use crate::node::napi_error::MapToNapiError;
use crate::node::napi_util::call_trampoline_func;
use java_rs::objects::object::GlobalJavaObject;
use napi::{
    check_status, sys, Callback, Env, JsFunction, JsObject, JsUnknown, NapiRaw, PropertyAttributes,
};
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;

pub(crate) unsafe extern "C" fn get_class_field(
    raw_env: sys::napi_env,
    cb_info: sys::napi_callback_info,
) -> sys::napi_value {
    call_trampoline_func(raw_env, cb_info, |ctx, data| {
        let name: &String = data.ok_or("data is null").map_napi_err()?;
        let this: JsObject = ctx.this()?;

        let proxy_obj: JsObject = this.get_named_property(CLASS_PROXY_PROPERTY)?;
        let instance_obj: JsObject = this.get_named_property(OBJECT_PROPERTY)?;
        let proxy: &Arc<JavaClassProxy> = ctx.env.unwrap(&proxy_obj)?;
        let obj: &GlobalJavaObject = ctx.env.unwrap(&instance_obj)?;

        let field = proxy.get_field_by_name(name.as_str()).map_napi_err()?;

        let res = field.get(obj).map_napi_err()?;
        let j_env = proxy.vm.attach_thread().map_napi_err()?;
        res.to_napi_value(&j_env, &ctx.env).map_napi_err()
    })
}

pub(crate) unsafe extern "C" fn set_class_field(
    raw_env: sys::napi_env,
    cb_info: sys::napi_callback_info,
) -> sys::napi_value {
    call_trampoline_func(raw_env, cb_info, |ctx, data| {
        let name: &String = data.ok_or("data is null").map_napi_err()?;
        let this: JsObject = ctx.this()?;
        let value: JsUnknown = ctx.get(0)?;

        let proxy_obj: JsObject = this.get_named_property(CLASS_PROXY_PROPERTY)?;
        let instance_obj: JsObject = this.get_named_property(OBJECT_PROPERTY)?;
        let proxy: &Arc<JavaClassProxy> = ctx.env.unwrap(&proxy_obj)?;
        let obj: &GlobalJavaObject = ctx.env.unwrap(&instance_obj)?;

        let field = proxy.get_field_by_name(name.as_str()).map_napi_err()?;

        let field_type = field.get_type();
        let j_env = proxy.vm.attach_thread().map_napi_err()?;
        let val = field_type
            .convert_to_java_value(&j_env, &ctx.env, value)
            .map_napi_err()?;

        field.set(obj, val).map_napi_err()?;
        ctx.env.get_undefined()
    })
}

pub(crate) unsafe extern "C" fn get_static_class_field(
    raw_env: sys::napi_env,
    cb_info: sys::napi_callback_info,
) -> sys::napi_value {
    call_trampoline_func(raw_env, cb_info, |ctx, data| {
        let name: &String = data.ok_or("data is null").map_napi_err()?;
        let this = ctx.this::<JsFunction>()?.coerce_to_object()?;

        let proxy_obj: JsObject = this.get_named_property(CLASS_PROXY_PROPERTY)?;
        let proxy: &Arc<JavaClassProxy> = ctx.env.unwrap(&proxy_obj)?;

        let field = proxy
            .get_static_field_by_name(name.as_str())
            .map_napi_err()?;

        let res = field.get_static().map_napi_err()?;
        let j_env = proxy.vm.attach_thread().map_napi_err()?;
        res.to_napi_value(&j_env, &ctx.env).map_napi_err()
    })
}

pub(crate) unsafe extern "C" fn set_static_class_field(
    raw_env: sys::napi_env,
    cb_info: sys::napi_callback_info,
) -> sys::napi_value {
    call_trampoline_func(raw_env, cb_info, |ctx, data| {
        let name: &String = data.ok_or("data is null").map_napi_err()?;
        let this = ctx.this::<JsFunction>()?.coerce_to_object()?;

        let proxy_obj: JsObject = this.get_named_property(CLASS_PROXY_PROPERTY)?;
        let proxy: &Arc<JavaClassProxy> = ctx.env.unwrap(&proxy_obj)?;

        let field = proxy
            .get_static_field_by_name(name.as_str())
            .map_napi_err()?;

        let field_type = field.get_type();
        let j_env = proxy.vm.attach_thread().map_napi_err()?;
        let val = field_type
            .convert_to_java_value(&j_env, &ctx.env, ctx.get(0)?)
            .map_napi_err()?;

        field.set_static(val).map_napi_err()?;
        ctx.env.get_undefined()
    })
}

pub(crate) trait PropertyWithData {
    fn create_property_with_data<T: 'static + Send + Sync>(
        &mut self,
        env: &Env,
        name: String,
        attributes: PropertyAttributes,
        getter: Option<Callback>,
        setter: Option<Callback>,
        data: T,
    ) -> napi::Result<()>;
}

impl PropertyWithData for JsObject {
    fn create_property_with_data<T: 'static + Send + Sync>(
        &mut self,
        env: &Env,
        name: String,
        attributes: PropertyAttributes,
        getter: Option<Callback>,
        setter: Option<Callback>,
        data: T,
    ) -> napi::Result<()> {
        let name = CString::new(name)?;
        let data = Box::into_raw(Box::new(data));

        self.add_finalizer(data, name.clone(), |ctx| unsafe {
            drop(Box::from_raw(ctx.value));
        })?;

        let descriptor = sys::napi_property_descriptor {
            utf8name: name.as_ptr(),
            name: ptr::null_mut(),
            method: None,
            getter,
            setter,
            value: ptr::null_mut(),
            attributes: attributes.into(),
            data: data as _,
        };

        check_status!(unsafe {
            sys::napi_define_properties(env.raw(), self.raw(), 1, vec![descriptor].as_ptr())
        })
    }
}
