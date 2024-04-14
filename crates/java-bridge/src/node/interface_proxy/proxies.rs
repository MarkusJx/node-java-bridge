use crate::node::interface_proxy::function_caller::FunctionCaller;
use crate::node::interface_proxy::types::{MethodMap, MethodsType, ProxiesType};
use crate::node::util::helpers::ResultType;
use lazy_static::lazy_static;
use napi::Env;
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

type DaemonProxiesType = HashMap<usize, (MethodsType, FunctionCaller)>;

lazy_static! {
    static ref PROXIES: Mutex<ProxiesType> = Mutex::new(HashMap::new());
    static ref DAEMON_PROXIES: Mutex<DaemonProxiesType> = Mutex::new(HashMap::new());
}

pub(in crate::node::interface_proxy) fn get_proxies<'a>() -> MutexGuard<'a, ProxiesType> {
    PROXIES.lock().unwrap()
}

pub(in crate::node::interface_proxy) fn get_daemon_proxies<'a>() -> MutexGuard<'a, DaemonProxiesType>
{
    DAEMON_PROXIES.lock().unwrap()
}

pub(in crate::node::interface_proxy) fn generate_proxy_id(
    proxies: &MutexGuard<ProxiesType>,
    daemon_proxies: &MutexGuard<DaemonProxiesType>,
) -> usize {
    let mut rng = rand::thread_rng();
    let mut id: usize = rng.gen();

    while proxies.contains_key(&id) || daemon_proxies.contains_key(&id) {
        id = rng.gen();
    }

    id
}

pub(in crate::node::interface_proxy) fn find_methods_by_id(
    id: usize,
    proxies: &MutexGuard<ProxiesType>,
    daemon_proxies: &MutexGuard<DaemonProxiesType>,
) -> ResultType<MethodMap> {
    if let Some(methods) = proxies.get(&id) {
        Ok(methods.lock().unwrap().clone())
    } else if let Some((methods, _)) = daemon_proxies.get(&id) {
        Ok(methods.lock().unwrap().clone())
    } else {
        Err(format!("No proxy with the id {} exists", id).into())
    }
}

pub(in crate::node::interface_proxy) fn remove_proxy(
    id: usize,
    keep_as_daemon: bool,
    proxies: &mut MutexGuard<ProxiesType>,
    daemon_proxies: &mut MutexGuard<DaemonProxiesType>,
    function_caller: Option<FunctionCaller>,
) {
    let removed = proxies.remove(&id);

    if keep_as_daemon && function_caller.is_some() && function_caller.as_ref().unwrap().is_alive() {
        if let Some(methods) = removed {
            daemon_proxies.insert(id, (methods, function_caller.unwrap()));
        }
    }
}

pub fn interface_proxy_exists() -> bool {
    !PROXIES.lock().unwrap().is_empty() || !DAEMON_PROXIES.lock().unwrap().is_empty()
}

/// Clears the list of daemon proxies.
#[napi]
#[allow(unused)]
pub fn clear_daemon_proxies(env: Env) -> napi::Result<()> {
    let mut proxies = DAEMON_PROXIES.lock().unwrap();
    for (_, (methods, function_caller)) in proxies.iter_mut() {
        function_caller.destroy(Some(env))?;
        methods.lock().unwrap().clear();
    }

    proxies.clear();
    Ok(())
}
