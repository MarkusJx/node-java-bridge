use crate::node::java_class_proxy::JavaClassProxy;
use crate::node::util::util::ResultType;
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
    ) -> ResultType<Arc<JavaClassProxy>> {
        if self.0.contains_key(class_name.as_str()) {
            Ok(self.0.get(class_name.as_str()).unwrap().clone())
        } else {
            trace!("Caching class proxy for {}", class_name);
            let proxy = Arc::new(JavaClassProxy::new(vm.clone(), class_name.clone())?);
            self.0.insert(class_name, proxy.clone());

            Ok(proxy)
        }
    }
}
