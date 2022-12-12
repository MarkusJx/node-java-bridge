use crate::node::java_type_ext::NapiToJava;
use crate::node::napi_error::MapToNapiError;
use java_rs::java_call_result::JavaCallResult;
use java_rs::java_env::JavaEnv;
use java_rs::java_type::JavaType;
use java_rs::objects::args::{JavaArg, JavaArgs};
use java_rs::traits::ToJavaValue;
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
