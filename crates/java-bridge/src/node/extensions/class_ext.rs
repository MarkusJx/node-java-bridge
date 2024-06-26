use crate::java::class_constructor::ClassConstructor;
use crate::java::class_method::ClassMethod;
use crate::node::extensions::java_type_ext::JsTypeEq;
use java_rs::java_type::{JavaType, Type};
use napi::{CallContext, Env, JsUnknown};

pub trait ArgumentMatch {
    fn arguments_match(&self, ctx: &CallContext, allow_objects: bool) -> napi::Result<bool>;
}

fn argument_matches(
    arg: &JavaType,
    value: JsUnknown,
    env: &Env,
    allow_objects: bool,
) -> napi::Result<bool> {
    Ok(arg.js_equals(value, env)? || (allow_objects && arg.type_enum() == Type::LangObject))
}

fn arguments_match(
    parameter_types: &[JavaType],
    ctx: &CallContext,
    allow_objects: bool,
) -> napi::Result<bool> {
    if ctx.length != parameter_types.len() {
        return Ok(false);
    }

    for (i, param) in parameter_types.iter().enumerate() {
        if !argument_matches(param, ctx.get(i)?, ctx.env, allow_objects)? {
            return Ok(false);
        }
    }

    Ok(true)
}

impl ArgumentMatch for ClassMethod {
    fn arguments_match(&self, ctx: &CallContext, allow_objects: bool) -> napi::Result<bool> {
        arguments_match(self.parameter_types(), ctx, allow_objects)
    }
}

impl ArgumentMatch for ClassConstructor {
    fn arguments_match(&self, ctx: &CallContext, allow_objects: bool) -> napi::Result<bool> {
        arguments_match(self.parameter_types(), ctx, allow_objects)
    }
}
