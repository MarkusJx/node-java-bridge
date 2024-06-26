/// Options for the interface proxies
#[napi(object)]
#[derive(Default)]
pub struct InterfaceProxyOptions {
    /// If true, the proxy will be kept as a daemon
    /// proxy after the interface has been destroyed
    pub keep_as_daemon: Option<bool>,
}
