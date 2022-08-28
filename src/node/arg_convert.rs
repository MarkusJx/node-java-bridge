use crate::jni::java_call_result::JavaCallResult;
use crate::jni::java_env::JavaEnv;
use crate::jni::java_type::JavaType;
use crate::jni::objects::args::{JavaArg, JavaArgs};
use crate::jni::traits::ToJavaValue;
use crate::node::java_type_ext::NapiToJava;
use crate::node::napi_error::MapToNapiError;
use napi::{CallContext, JsUnknown};

pub fn call_context_to_java_args<'a>(
    ctx: &'a CallContext,
    signatures: &'a Vec<JavaType>,
    env: &'a JavaEnv<'a>,
) -> napi::Result<Vec<JavaCallResult>> {
    let mut res: Vec<JavaCallResult> = vec![];
    for i in 0..signatures.len() {
        let js_value: JsUnknown = ctx.get(i)?;
        let signature = signatures.get(i).unwrap();
        res.insert(
            i,
            signature
                .convert_to_java_value(env, ctx.env, js_value)
                .map_napi_err()?,
        );
    }

    Ok(res)
}

pub fn call_results_to_args(args: &Vec<JavaCallResult>) -> JavaArgs {
    args.iter()
        .map(|arg| Box::<&dyn ToJavaValue>::new(arg))
        .collect::<Vec<JavaArg>>()
}
