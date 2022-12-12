use crate::node::arg_convert::{call_context_to_java_args, call_results_to_args};
use crate::node::java::Java;
use crate::node::java_call_result_ext::ToNapiValue;
use crate::node::java_class_proxy::JavaClassProxy;
use crate::node::java_type_ext::NapiToJava;
use crate::node::napi_error::MapToNapiError;
use futures::future;
use java_rs::java_call_result::JavaCallResult;
use java_rs::java_type::JavaType;
use java_rs::objects::class::GlobalJavaClass;
use java_rs::objects::object::GlobalJavaObject;
use napi::{
    CallContext, Env, JsBoolean, JsFunction, JsObject, JsString, JsUnknown, Property,
    PropertyAttributes, Status,
};
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
        let res = method.call_static(args_ref).map_napi_err()?;

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
                method.call_static(args_ref).map_napi_err()
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

        let result = method.call(&obj, args_ref).map_napi_err()?;
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
                Ok(method.call(&obj, args_ref).map_napi_err()?)
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
    let instance = constructor.new_instance(args_ref).map_napi_err()?;

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
            constructor.new_instance(args_ref).map_napi_err()
        }),
        move |env, instance| JavaClassInstance::from_existing(proxy.clone(), env, instance),
    )
}

/// An object storing information about java class fields
#[napi(object)]
pub struct ClassFieldInfo {
    /// The name of the field
    pub name: String,
    /// Whether the field is final
    pub is_final: bool,
}

/// Get the fields of a java class
///
/// @param proxy the java class proxy
/// @param getStatic whether to get only static fields or not
/// @returns an array of field info objects
#[napi]
fn get_class_fields(
    env: Env,
    proxy: JsObject,
    get_static: bool,
) -> napi::Result<Vec<ClassFieldInfo>> {
    let proxy: &Arc<JavaClassProxy> = env.unwrap(&proxy)?;

    let mut res = vec![];
    let fields = if get_static {
        &proxy.static_fields
    } else {
        &proxy.fields
    };

    for field in fields.iter() {
        res.push(ClassFieldInfo {
            name: field.0.to_string(),
            is_final: field.1.is_final(),
        });
    }

    Ok(res)
}

/// Get the value of a non-static java class field
///
/// @param cls the class to get the field from
/// @param name the name of the field to get
/// @returns the value of the field
#[napi]
fn get_field(env: Env, cls: JsObject, name: JsString) -> napi::Result<JsUnknown> {
    let proxy_obj: JsObject = cls.get_named_property(CLASS_PROXY_PROPERTY)?;
    let instance_obj: JsObject = cls.get_named_property(OBJECT_PROPERTY)?;
    let proxy: &Arc<JavaClassProxy> = env.unwrap(&proxy_obj)?;
    let obj: &GlobalJavaObject = env.unwrap(&instance_obj)?;
    let name_u8 = name.into_utf8()?;
    let name = name_u8.as_str()?;

    let field = proxy.get_field_by_name(name).map_napi_err()?;

    let res = field.get(obj).map_napi_err()?;
    let j_env = proxy.vm.attach_thread().map_napi_err()?;
    res.to_napi_value(&j_env, &env).map_napi_err()
}

/// Set the value of a java field
///
/// @param cls the java class instance
/// @param name the field name
/// @param value the value to set the field to
#[napi]
fn set_field(env: Env, cls: JsObject, name: JsString, value: JsUnknown) -> napi::Result<()> {
    let proxy_obj: JsObject = cls.get_named_property(CLASS_PROXY_PROPERTY)?;
    let instance_obj: JsObject = cls.get_named_property(OBJECT_PROPERTY)?;
    let proxy: &Arc<JavaClassProxy> = env.unwrap(&proxy_obj)?;
    let obj: &GlobalJavaObject = env.unwrap(&instance_obj)?;
    let name_u8 = name.into_utf8()?;
    let name = name_u8.as_str()?;

    let field = proxy.get_field_by_name(name).map_napi_err()?;

    let field_type = field.get_type();
    let j_env = proxy.vm.attach_thread().map_napi_err()?;
    let val = field_type
        .convert_to_java_value(&j_env, &env, value)
        .map_napi_err()?;
    field.set(obj, val).map_napi_err()
}

/// Get the value of a static java class field
#[napi]
fn get_static_field(env: Env, cls: JsObject, name: JsString) -> napi::Result<JsUnknown> {
    let proxy_obj: JsObject = cls.get_named_property(CLASS_PROXY_PROPERTY)?;
    let proxy: &Arc<JavaClassProxy> = env.unwrap(&proxy_obj)?;
    let name_u8 = name.into_utf8()?;
    let name = name_u8.as_str()?;

    let field = proxy.get_static_field_by_name(name).map_napi_err()?;

    let res = field.get_static().map_napi_err()?;
    let j_env = proxy.vm.attach_thread().map_napi_err()?;
    res.to_napi_value(&j_env, &env).map_napi_err()
}

#[napi]
fn set_static_field(env: Env, cls: JsObject, name: JsString, value: JsUnknown) -> napi::Result<()> {
    let proxy_obj: JsObject = cls.get_named_property(CLASS_PROXY_PROPERTY)?;
    let proxy: &Arc<JavaClassProxy> = env.unwrap(&proxy_obj)?;
    let name_u8 = name.into_utf8()?;
    let name = name_u8.as_str()?;

    let field = proxy.get_static_field_by_name(name).map_napi_err()?;

    let field_type = field.get_type();
    let j_env = proxy.vm.attach_thread().map_napi_err()?;
    let val = field_type
        .convert_to_java_value(&j_env, &env, value)
        .map_napi_err()?;

    field.set_static(val).map_napi_err()
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
        object: class.to_object(),
        signature: JavaType::new("java.lang.Class".to_string(), false),
    };

    res.to_napi_value(&j_env, &ctx.env)
        .map_napi_err()?
        .coerce_to_object()
}
