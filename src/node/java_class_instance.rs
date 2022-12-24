use crate::node::extensions::java_call_result_ext::ToNapiValue;
use crate::node::extensions::property_with_data::{DefinePropertiesWithData, PropertyWithData};
use crate::node::helpers::arg_convert::{call_context_to_java_args, call_results_to_args};
use crate::node::helpers::napi_error::MapToNapiError;
use crate::node::java::Java;
use crate::node::java_class_field::{
    get_static_class_field, set_class_field, set_static_class_field,
};
use crate::node::java_class_proxy::JavaClassProxy;
use futures::future;
use java_rs::java_call_result::JavaCallResult;
use java_rs::java_type::JavaType;
use java_rs::objects::class::GlobalJavaClass;
use java_rs::objects::object::GlobalJavaObject;
use napi::{
    CallContext, Callback, Env, JsBoolean, JsFunction, JsObject, JsUnknown, Property,
    PropertyAttributes, Status,
};
use std::ops::Not;
use std::sync::Arc;

pub const CLASS_PROXY_PROPERTY: &str = "class.proxy";
pub const OBJECT_PROPERTY: &str = "class.object";

pub struct JavaClassInstance {}

impl JavaClassInstance {
    pub fn create_class_instance(
        env: &Env,
        proxy: Arc<JavaClassProxy>,
    ) -> napi::Result<JsFunction> {
        let mut proxy_obj = env.create_object()?;
        env.wrap(&mut proxy_obj, proxy.clone())?;

        let mut constructor = env
            .define_class("JavaClass", constructor, &[])?
            .coerce_to_object()?;

        constructor.set_named_property(CLASS_PROXY_PROPERTY, proxy_obj)?;
        constructor.set_named_property(
            "newInstanceAsync",
            env.create_function("newInstanceAsync", new_instance)?,
        )?;

        constructor.define_properties(&[Property::new("class")?
            .with_getter(get_class_field)
            .with_property_attributes(PropertyAttributes::Enumerable)])?;

        for method in &proxy.static_methods {
            let name = method.0.clone();
            let name_cpy = name.clone();
            let name_sync = name.clone() + "Sync";

            constructor.set_named_property(
                name_sync.clone().as_str(),
                env.create_function_from_closure(
                    name_sync.clone().as_str(),
                    move |ctx: CallContext| -> napi::Result<JsUnknown> {
                        Self::call_static_method(&ctx, &name_cpy)
                    },
                )?,
            )?;

            constructor.set_named_property(
                name.clone().as_str(),
                env.create_function_from_closure(
                    name.clone().as_str(),
                    move |ctx: CallContext| -> napi::Result<JsObject> {
                        Self::call_static_method_async(&ctx, &name)
                    },
                )?,
            )?;
        }

        constructor.define_properties_with_data(
            env,
            (&proxy.static_fields)
                .into_iter()
                .map(|(name, field)| {
                    Ok(PropertyWithData::new(name.clone(), name.clone())?
                        .with_attributes(PropertyAttributes::Enumerable)
                        .with_getter_and_setter(
                            Some(get_static_class_field),
                            field
                                .is_final()
                                .not()
                                .then(|| set_static_class_field as Callback),
                        ))
                })
                .collect::<napi::Result<Vec<_>>>()?
                .as_ref(),
        )?;

        JsFunction::try_from(constructor.into_unknown())
    }

    pub fn from_existing(
        proxy: Arc<JavaClassProxy>,
        env: &Env,
        instance: GlobalJavaObject,
    ) -> napi::Result<JsUnknown> {
        let mut this = env.create_object()?;
        let mut proxy_obj = env.create_object()?;
        env.wrap(&mut proxy_obj, proxy.clone())?;
        this.set_named_property(CLASS_PROXY_PROPERTY, proxy_obj)?;

        JavaClassInstance::add_class_methods(env, &mut this, &proxy, instance)?;
        Ok(this.into_unknown())
    }

    fn add_class_methods(
        env: &Env,
        this: &mut JsObject,
        proxy: &Arc<JavaClassProxy>,
        instance: GlobalJavaObject,
    ) -> napi::Result<()> {
        let mut instance_obj = env.create_object()?;
        env.wrap(&mut instance_obj, instance)?;
        this.set_named_property(OBJECT_PROPERTY, instance_obj)?;

        for method in &proxy.methods {
            let name = method.0.clone();
            let name_cpy = name.clone();
            let name_sync = name.clone() + "Sync";

            this.set_named_property(
                name_sync.clone().as_str(),
                env.create_function_from_closure(
                    name_sync.clone().as_str(),
                    move |ctx: CallContext| -> napi::Result<JsUnknown> {
                        Self::call_method(&ctx, &name_cpy)
                    },
                )?,
            )?;

            this.set_named_property(
                name.clone().as_str(),
                env.create_function_from_closure(
                    name.clone().as_str(),
                    move |ctx: CallContext| -> napi::Result<JsObject> {
                        Self::call_method_async(&ctx, &name)
                    },
                )?,
            )?;
        }

        this.define_properties_with_data(
            env,
            (&proxy.fields)
                .into_iter()
                .map(|(name, field)| {
                    Ok(PropertyWithData::new(name.clone(), name.clone())?
                        .with_attributes(PropertyAttributes::Enumerable)
                        .with_getter_and_setter(
                            Some(crate::node::java_class_field::get_class_field),
                            field.is_final().not().then(|| set_class_field as Callback),
                        ))
                })
                .collect::<napi::Result<Vec<_>>>()?
                .as_ref(),
        )?;

        if !proxy.methods.contains_key("instanceOf") {
            this.set_named_property(
                "instanceOf",
                env.create_function_from_closure(
                    "instanceOf",
                    |ctx: CallContext| -> napi::Result<JsBoolean> {
                        let proxy = Self::get_class_proxy(&ctx, false)?;
                        let env = proxy.vm.attach_thread().map_napi_err()?;
                        let res = Java::_is_instance_of(env, ctx.env, ctx.this()?, ctx.get(0)?)?;

                        ctx.env.get_boolean(res)
                    },
                )?,
            )?;
        }

        Ok(())
    }

    fn get_class_proxy<'a>(
        ctx: &'a CallContext,
        is_static: bool,
    ) -> napi::Result<&'a Arc<JavaClassProxy>> {
        let this: JsObject = if is_static {
            ctx.this::<JsFunction>()?.coerce_to_object()?
        } else {
            ctx.this()?
        };
        let proxy_obj: JsObject = this.get_named_property(CLASS_PROXY_PROPERTY)?;
        Ok(ctx.env.unwrap(&proxy_obj)?)
    }

    fn get_object<'a>(ctx: &'a CallContext) -> napi::Result<&'a GlobalJavaObject> {
        let this: JsObject = ctx.this()?;
        let object_obj: JsObject = this.get_named_property(OBJECT_PROPERTY)?;
        Ok(ctx.env.unwrap(&object_obj)?)
    }

    fn call_static_method(ctx: &CallContext, name: &String) -> napi::Result<JsUnknown> {
        let proxy = Self::get_class_proxy(ctx, true)?;
        let method = proxy
            .find_matching_method(ctx, name, true, false)
            .or_else(|_| proxy.find_matching_method(ctx, name, true, true))
            .map_napi_err()?;
        let env = proxy.vm.attach_thread().map_napi_err()?;
        let args = call_context_to_java_args(ctx, method.parameter_types(), &env)?;
        let args_ref = call_results_to_args(&args);
        let res = method.call_static(args_ref.as_slice()).map_napi_err()?;

        res.to_napi_value(&env, ctx.env).map_napi_err()
    }

    fn call_static_method_async(ctx: &CallContext, name: &String) -> napi::Result<JsObject> {
        let proxy = Self::get_class_proxy(ctx, true)?.clone();
        let method = proxy
            .find_matching_method(ctx, name, true, false)
            .or_else(|_| proxy.find_matching_method(ctx, name, true, true))
            .map_napi_err()?
            .clone();
        let env = proxy.vm.attach_thread().map_napi_err()?;
        let args = call_context_to_java_args(ctx, method.parameter_types(), &env)?;

        ctx.env.execute_tokio_future(
            futures::future::lazy(move |_| {
                let args_ref = call_results_to_args(&args);
                method.call_static(args_ref.as_slice()).map_napi_err()
            }),
            move |&mut env, res| {
                let j_env = proxy.vm.attach_thread().map_napi_err()?;
                res.to_napi_value(&j_env, &env).map_napi_err()
            },
        )
    }

    fn call_method(ctx: &CallContext, name: &String) -> napi::Result<JsUnknown> {
        let proxy = Self::get_class_proxy(ctx, false)?;
        let method = proxy
            .find_matching_method(ctx, name, false, false)
            .or_else(|_| proxy.find_matching_method(ctx, name, false, true))
            .map_napi_err()?;
        let obj = Self::get_object(ctx)?;

        let env = proxy.vm.attach_thread().map_napi_err()?;
        let args = call_context_to_java_args(ctx, method.parameter_types(), &env)?;
        let args_ref = call_results_to_args(&args);

        let result = method.call(&obj, args_ref.as_slice()).map_napi_err()?;
        result.to_napi_value(&env, ctx.env).map_napi_err()
    }

    fn call_method_async(ctx: &CallContext, name: &String) -> napi::Result<JsObject> {
        let proxy = Self::get_class_proxy(ctx, false)?.clone();
        let method = proxy
            .find_matching_method(ctx, name, false, false)
            .or_else(|_| proxy.find_matching_method(ctx, name, false, true))
            .map_napi_err()?
            .clone();
        let obj = Self::get_object(ctx)?.clone();
        let env = proxy.vm.attach_thread().map_napi_err()?;
        let args = call_context_to_java_args(ctx, method.parameter_types(), &env)?;

        ctx.env.execute_tokio_future(
            futures::future::lazy(move |_| {
                let args_ref = call_results_to_args(&args);
                Ok(method.call(&obj, args_ref.as_slice()).map_napi_err()?)
            }),
            move |&mut env, res| {
                let j_env = proxy.vm.attach_thread().map_napi_err()?;
                res.to_napi_value(&j_env, &env).map_napi_err()
            },
        )
    }
}

#[js_function(255)]
fn constructor(ctx: CallContext) -> napi::Result<JsUnknown> {
    let new_target_func = ctx.get_new_target::<JsFunction>();
    if new_target_func.is_err() {
        return Err(napi::Error::new(Status::Unknown, "Could not get the new target function, did you forget to add the 'new' keyword before this constructor call?".to_string()));
    }

    let new_target: JsObject = new_target_func.unwrap().coerce_to_object()?;
    let mut this: JsObject = ctx.this()?;
    let proxy_obj: JsObject = new_target.get_named_property(CLASS_PROXY_PROPERTY)?;
    let proxy: &Arc<JavaClassProxy> = ctx.env.unwrap(&proxy_obj)?;

    let constructor = proxy
        .find_matching_constructor(&ctx, false)
        .or_else(|_| proxy.find_matching_constructor(&ctx, true))
        .map_napi_err()?;
    let env = proxy.vm.attach_thread().map_napi_err()?;

    let args = call_context_to_java_args(&ctx, constructor.parameter_types(), &env)?;
    let args_ref = call_results_to_args(&args);
    let instance = constructor
        .new_instance(args_ref.as_slice())
        .map_napi_err()?;

    this.set_named_property(CLASS_PROXY_PROPERTY, proxy_obj)?;

    JavaClassInstance::add_class_methods(ctx.env, &mut this, proxy, instance)?;
    Ok(ctx.env.get_undefined()?.into_unknown())
}

#[js_function(255)]
fn new_instance(ctx: CallContext) -> napi::Result<JsObject> {
    let proxy = JavaClassInstance::get_class_proxy(&ctx, true)?.clone();
    let constructor = proxy
        .find_matching_constructor(&ctx, false)
        .or_else(|_| proxy.find_matching_constructor(&ctx, true))
        .map_napi_err()?
        .clone();
    let env = proxy.vm.attach_thread().map_napi_err()?;
    let args = call_context_to_java_args(&ctx, constructor.parameter_types(), &env)?;

    ctx.env.execute_tokio_future(
        future::lazy(move |_| {
            let args_ref = call_results_to_args(&args);
            constructor.new_instance(args_ref.as_slice()).map_napi_err()
        }),
        move |env, instance| JavaClassInstance::from_existing(proxy.clone(), env, instance),
    )
}

#[js_function(0)]
fn get_class_field(ctx: CallContext) -> napi::Result<JsObject> {
    let cls: JsFunction = ctx.this()?;
    let proxy_obj: JsObject = cls
        .coerce_to_object()?
        .get_named_property(CLASS_PROXY_PROPERTY)?;
    let proxy: &Arc<JavaClassProxy> = ctx.env.unwrap(&proxy_obj)?;

    let j_env = proxy.vm.attach_thread().map_napi_err()?;
    let class = GlobalJavaClass::by_name(proxy.class_name.as_str(), &j_env).map_napi_err()?;

    let res = JavaCallResult::Object {
        object: class.into_object(),
        signature: JavaType::new("java.lang.Class".to_string(), false),
    };

    res.to_napi_value(&j_env, &ctx.env)
        .map_napi_err()?
        .coerce_to_object()
}
