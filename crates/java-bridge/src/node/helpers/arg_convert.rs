use crate::node::extensions::java_type_ext::NapiToJava;
use crate::node::helpers::napi_error::MapToNapiError;
use java_rs::java_call_result::JavaCallResult;
use java_rs::java_env::JavaEnv;
use java_rs::java_type::JavaType;
use java_rs::objects::args::{AsJavaArg, JavaArg};
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
                .map_napi_err(Some(*ctx.env))?,
        );
    }

    Ok(res)
}

pub fn call_results_to_args(args: &Vec<JavaCallResult>) -> Vec<JavaArg> {
    args.iter()
        .map(|arg| arg.as_arg())
        .collect::<Vec<JavaArg>>()
}
