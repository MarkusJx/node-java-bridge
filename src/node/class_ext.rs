use crate::node::java_type_ext::JsTypeEq;
use java_rs::class_constructor::ClassConstructor;
use java_rs::class_method::ClassMethod;
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
    parameter_types: &Vec<JavaType>,
    ctx: &CallContext,
    allow_objects: bool,
) -> napi::Result<bool> {
    if ctx.length != parameter_types.len() {
        return Ok(false);
    }

    for i in 0..ctx.length {
        if !argument_matches(&parameter_types[i], ctx.get(i)?, ctx.env, allow_objects)? {
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
