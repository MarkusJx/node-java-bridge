use std::sync::{Mutex, MutexGuard};

use crate::node::java_class_instance::EMPTY_STRING;
use lazy_static::lazy_static;
use smart_default::SmartDefault;

lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new(Config::default());
}

/// Configuration for the Java class proxy.
///
/// @since 2.4.0
#[napi(object)]
pub struct ClassConfiguration {
    /// If true, the event loop will be run when an interface proxy is active.
    pub run_event_loop_when_interface_proxy_is_active: Option<bool>,
    /// If true, the custom inspect method will be used to display the object in the console.
    pub custom_inspect: Option<bool>,
    /// The suffix to use for synchronous methods.
    pub sync_suffix: Option<String>,
    /// The suffix to use for asynchronous methods.
    pub async_suffix: Option<String>,
}

impl TryFrom<ClassConfiguration> for Config {
    type Error = napi::Error;

    fn try_from(value: ClassConfiguration) -> napi::Result<Self> {
        let config = Config::get();
        let async_suffix = config.async_suffix.as_ref();
        let sync_suffix = config.sync_suffix.as_ref();

        if value
            .sync_suffix
            .as_ref()
            .or(sync_suffix)
            .unwrap_or(&EMPTY_STRING)
            == value
                .async_suffix
                .as_ref()
                .or(async_suffix)
                .unwrap_or(&EMPTY_STRING)
        {
            return Err(napi::Error::from_reason(
                "syncSuffix and asyncSuffix cannot be the same",
            ));
        }

        Ok(Self {
            run_event_loop_when_interface_proxy_is_active: value
                .run_event_loop_when_interface_proxy_is_active
                .unwrap_or(config.run_event_loop_when_interface_proxy_is_active),
            custom_inspect: value.custom_inspect.unwrap_or(config.custom_inspect),
            sync_suffix: value.sync_suffix.or(sync_suffix.map(|s| s.clone())),
            async_suffix: value.async_suffix.or(async_suffix.map(|s| s.clone())),
        })
    }
}

/// Configuration for the Java class proxy.
///
/// @since 2.4.0
#[napi(object)]
#[derive(SmartDefault, Clone, Eq, PartialEq)]
pub struct Config {
    /// If true, the event loop will be run when an interface proxy is active.
    ///
    /// @since 2.2.3
    #[default(false)]
    pub run_event_loop_when_interface_proxy_is_active: bool,
    /// If true, the custom inspect method will be used to display the object in the console.
    ///
    /// @since 2.4.0
    #[default(false)]
    pub custom_inspect: bool,
    /// The suffix to use for synchronous methods.
    ///
    /// @since 2.4.0
    #[default(Some("Sync".to_string()))]
    pub sync_suffix: Option<String>,
    /// The suffix to use for asynchronous methods.
    ///
    /// @since 2.4.0
    #[default(None)]
    pub async_suffix: Option<String>,
}

impl Config {
    pub fn get<'a>() -> MutexGuard<'a, Config> {
        CONFIG.lock().unwrap()
    }
}
