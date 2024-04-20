use crate::node::helpers::napi_error::MapToNapiError;
use java_rs::objects::java_object::JavaObject;
use java_rs::objects::object::GlobalJavaObject;
use napi::Env;

pub struct FunctionCaller {
    instance: Option<GlobalJavaObject>,
}

impl FunctionCaller {
    pub fn new(instance: GlobalJavaObject) -> Self {
        Self {
            instance: Some(instance),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.instance.is_some()
    }

    pub fn is_dead(&self) -> bool {
        self.instance.is_none()
    }

    pub fn destroy(&mut self, env: Option<Env>) -> napi::Result<()> {
        if let Some(instance) = self.instance.as_ref() {
            let java_env = instance.get_vm().attach_thread().map_napi_err(env)?;
            let java_class = java_env
                .get_object_class(JavaObject::from(instance))
                .map_napi_err(None)?;
            let destruct = java_class
                .get_void_method("destruct", "()V")
                .map_napi_err(env)?;
            destruct
                .call(JavaObject::from(instance), &[])
                .map_napi_err(env)?;

            self.instance.take();
        }

        Ok(())
    }

    pub fn move_to(&mut self) -> Option<FunctionCaller> {
        self.instance.take().map(FunctionCaller::new)
    }
}

impl Drop for FunctionCaller {
    fn drop(&mut self) {
        if let Err(e) = self.destroy(None) {
            eprintln!("Error while dropping FunctionCaller: {}", e);
        }
    }
}
