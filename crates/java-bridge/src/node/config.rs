use std::sync::{Mutex, MutexGuard};

use lazy_static::lazy_static;
use smart_default::SmartDefault;

use crate::node::util::traits::UnwrapOrEmpty;

lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new(Config::default());
}

/// Configuration for the Java class proxy.
///
/// @since 2.4.0
#[napi(object)]
pub struct ClassConfiguration {
    /// If true, the event loop will be run when an interface proxy is active.
    /// If not specified, the value from the global configuration will be used.
    pub run_event_loop_when_interface_proxy_is_active: Option<bool>,
    /// If true, the custom inspect method will be used to display the object in the console.
    /// If not specified, the value from the global configuration will be used.
    pub custom_inspect: Option<bool>,
    /// The suffix to use for synchronous methods.
    /// Set this value to an empty string to disable the suffix.
    /// The default value is "Sync".
    /// Setting this value to the same value as asyncSuffix will result in an error.
    /// If not specified, the value from the global configuration will be used.
    pub sync_suffix: Option<String>,
    /// The suffix to use for asynchronous methods.
    /// Set this value to an empty string to disable the suffix.
    /// The default value is an empty string.
    /// Setting this value to the same value as syncSuffix will result in an error.
    /// If not specified, the value from the global configuration will be used.
    pub async_suffix: Option<String>,
}

impl TryFrom<ClassConfiguration> for Config {
    type Error = napi::Error;

    fn try_from(value: ClassConfiguration) -> napi::Result<Self> {
        let config = Config::get();
        let async_suffix = config.async_suffix.as_ref();
        let sync_suffix = config.sync_suffix.as_ref();

        if value.sync_suffix.as_ref().or(sync_suffix).unwrap_or_empty()
            == value
                .async_suffix
                .as_ref()
                .or(async_suffix)
                .unwrap_or_empty()
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
    /// Set this value to an empty string to disable the suffix.
    /// The default value is "Sync".
    /// Setting this value to the same value as asyncSuffix will result in an error.
    ///
    /// @since 2.4.0
    #[default(Some("Sync".to_string()))]
    pub sync_suffix: Option<String>,
    /// The suffix to use for asynchronous methods.
    /// Set this value to an empty string to disable the suffix.
    /// The default value is an empty string.
    /// Setting this value to the same value as syncSuffix will result in an error.
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
