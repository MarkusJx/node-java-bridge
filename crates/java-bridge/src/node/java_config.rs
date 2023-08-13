use crate::node::config::Config;
use crate::node::util::traits::UnwrapOrEmpty;

/// Configuration options for the java bridge.
///
/// As of version 2.4.0, the options are cached inside the class proxy cache.
/// This means that changing the options will not affect any class proxies
/// that have already been created by importing a class using {@link importClass}
/// or {@link importClassAsync}. You must clear the class proxy cache using the
/// {@link clearClassProxies} method in order to apply the new options to all
/// classes imported at a later date. This does not affect already instantiated
/// or imported classes.
///
/// Do not instantiate this class. Use the {@link default.config} instance instead.
///
/// @since 2.2.3
#[napi]
pub struct JavaConfig;

#[napi]
impl JavaConfig {
    /// Do not instantiate this class.
    /// Use the {@link default.config} instance instead.
    #[napi(constructor)]
    pub fn new() -> Self {
        Self
    }

    /// **Experimental Feature**
    ///
    /// Set whether to run the event loop when an interface proxy is active.
    /// This is disabled by default. Enabling this will cause the bridge
    /// to run the event loop when an interface proxy either as direct
    /// proxy or as daemon proxy is active. This is only required if the
    /// proxied method calls back into the javascript process in the same thread.
    /// If the proxy is used either in an async method or in a different thread,
    /// this is not required.
    ///
    /// **Note:** Enabling this may cause the application to crash. Use with caution.
    ///
    /// @since 2.2.3
    /// @experimental
    /// @param value whether to run the event loop when an interface proxy is active
    #[napi(setter)]
    pub fn set_run_event_loop_when_interface_proxy_is_active(&self, value: bool) {
        Config::get().run_event_loop_when_interface_proxy_is_active = value;
    }

    /// **Experimental Feature**
    ///
    /// Get whether to run the event loop when an interface proxy is active.
    /// @since 2.2.3
    /// @experimental
    #[napi(getter)]
    pub fn get_run_event_loop_when_interface_proxy_is_active(&self) -> bool {
        Config::get().run_event_loop_when_interface_proxy_is_active
    }

    /// Whether to add custom inspect methods to java objects.
    /// This is disabled by default.
    /// This allows console.log to print java objects in a more readable way
    /// using the `toString` method of the java object.
    ///
    /// @since 2.4.0
    /// @param value whether to add custom inspect methods to java objects
    #[napi(setter)]
    pub fn set_custom_inspect(&self, value: bool) {
        Config::get().custom_inspect = value;
    }

    /// Get whether to add custom inspect methods to java objects.
    ///
    /// @since 2.4.0
    /// @returns whether to add custom inspect methods to java objects
    #[napi(getter)]
    pub fn get_custom_inspect(&self) -> bool {
        Config::get().custom_inspect
    }

    /// Set the suffix for synchronous methods.
    /// This is `Sync` by default.
    /// Pass `null` or an empty string to disable the suffix.
    /// This must not be the same as the {@link asyncSuffix}.
    ///
    /// # Example
    /// ```ts
    /// import { config, clearClassProxies } from 'java-bridge';
    ///
    /// // Set the async suffix in order to prevent errors
    /// config.asyncSuffix = 'Async';
    /// // Set the sync suffix to an empty string
    /// config.syncSuffix = '';
    /// // This would do the same
    /// config.syncSuffix = null;
    ///
    /// // Clear the class proxy cache
    /// clearClassProxies();
    ///
    /// // Import the class
    /// const ArrayList = importClass('java.util.ArrayList');
    ///
    /// // Create a new instance
    /// const list = new ArrayList();
    ///
    /// // Call the method
    /// list.add('Hello World!');
    ///
    /// // Async methods now have the 'Async' suffix
    /// await list.addAsync('Hello World!');
    /// ```
    ///
    /// @see asyncSuffix
    /// @since 2.4.0
    /// @param value the suffix to use for synchronous methods
    #[napi(setter, ts_args_type = "value: string | undefined | null")]
    pub fn set_sync_suffix(&self, value: Option<String>) -> napi::Result<()> {
        if value.unwrap_or_empty() == Config::get().async_suffix.unwrap_or_empty() {
            Err(napi::Error::from_reason(
                "syncSuffix and asyncSuffix cannot be the same",
            ))
        } else {
            Config::get().sync_suffix = value;
            Ok(())
        }
    }

    /// Get the suffix for synchronous methods.
    ///
    /// @since 2.4.0
    #[napi(getter)]
    pub fn get_sync_suffix(&self) -> Option<String> {
        Config::get().sync_suffix.clone()
    }

    /// Set the suffix for asynchronous methods.
    /// This is `Async` by default.
    /// Pass `null` or an empty string to disable the suffix.
    /// This must not be the same as the {@link syncSuffix}.
    ///
    /// @see syncSuffix
    /// @since 2.4.0
    /// @param value the suffix to use for asynchronous methods
    #[napi(setter, ts_args_type = "value: string | undefined | null")]
    pub fn set_async_suffix(&self, value: Option<String>) -> napi::Result<()> {
        if value.unwrap_or_empty() == Config::get().sync_suffix.unwrap_or_empty() {
            Err(napi::Error::from_reason(
                "syncSuffix and asyncSuffix cannot be the same",
            ))
        } else {
            Config::get().async_suffix = value;
            Ok(())
        }
    }

    /// Get the suffix for asynchronous methods.
    ///
    /// @since 2.4.0
    #[napi(getter)]
    pub fn get_async_suffix(&self) -> Option<String> {
        Config::get().async_suffix.clone()
    }

    /// Override the whole config.
    /// If you want to change only a single field, use the static setters instead.
    ///
    /// @since 2.4.0
    /// @param value the config to use
    #[napi(setter)]
    pub fn set_config(&self, value: Config) -> napi::Result<()> {
        if value.async_suffix.unwrap_or_empty() == value.sync_suffix.unwrap_or_empty() {
            Err(napi::Error::from_reason(
                "syncSuffix and asyncSuffix cannot be the same",
            ))
        } else {
            Config::get().clone_from(&value);
            Ok(())
        }
    }

    /// Get the current config.
    ///
    /// @since 2.4.0
    #[napi(getter)]
    pub fn get_config(&self) -> Config {
        Config::get().clone()
    }

    /// Reset the config to the default values.
    ///
    /// @since 2.4.0
    #[napi]
    pub fn reset(&self) {
        Config::get().clone_from(&Config::default());
    }
}
