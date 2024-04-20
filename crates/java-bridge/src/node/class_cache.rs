use crate::node::config::{ClassConfiguration, Config};
use crate::node::java_class_proxy::JavaClassProxy;
use crate::node::util::helpers::ResultType;
use java_rs::java_vm::JavaVM;
use java_rs::trace;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct ClassCache(HashMap<String, Arc<JavaClassProxy>>);

impl ClassCache {
    pub fn get_class_proxy(
        &mut self,
        vm: &JavaVM,
        class_name: String,
        config: Option<ClassConfiguration>,
    ) -> ResultType<Arc<JavaClassProxy>> {
        let config = config
            .map(|c| c.try_into())
            .map_or(Ok(None), |c| c.map(Some))?;

        if let Some(proxy) = self.0.get(&class_name) {
            if let Some(cfg) = config {
                if proxy.config != cfg {
                    return Ok(Arc::new(JavaClassProxy::new(
                        vm.clone(),
                        class_name.clone(),
                        Some(cfg),
                    )?));
                }
            }

            Ok(proxy.clone())
        } else {
            if let Some(cfg) = config.as_ref() {
                if !Config::get().eq(cfg) {
                    return Ok(Arc::new(JavaClassProxy::new(
                        vm.clone(),
                        class_name.clone(),
                        Some(cfg.clone()),
                    )?));
                }
            }

            trace!("Caching class proxy for {}", class_name);
            let proxy = Arc::new(JavaClassProxy::new(vm.clone(), class_name.clone(), config)?);
            self.0.insert(class_name, proxy.clone());

            Ok(proxy)
        }
    }

    #[allow(unused)]
    pub fn clear(&mut self) {
        self.0.clear();
    }
}
