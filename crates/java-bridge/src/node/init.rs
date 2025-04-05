use crate::node::helpers::napi_error::NapiError;
use napi::bindgen_prelude::create_custom_tokio_runtime;
use napi::tokio::runtime::Builder;
use std::sync::OnceLock;

static INIT_RESULT: OnceLock<anyhow::Result<()>> = OnceLock::new();

#[napi::module_init]
fn init() {
    INIT_RESULT.get_or_init(|| {
        let mut builder = Builder::new_multi_thread();
        builder.enable_all();
        if let Ok(cpus) = std::env::var("JAVA_BRIDGE_THREAD_POOL_SIZE") {
            builder.worker_threads(cpus.parse()?);
        }

        create_custom_tokio_runtime(builder.build()?);

        Ok(())
    });
}

pub fn check_init_result() -> napi::Result<()> {
    if let Some(res) = INIT_RESULT.get() {
        match res {
            Ok(()) => Ok(()),
            Err(e) => Err(
                NapiError::from(format!("Failed to initialize tokio runtime: {}", e)).into_napi(),
            ),
        }
    } else {
        Ok(())
    }
}
