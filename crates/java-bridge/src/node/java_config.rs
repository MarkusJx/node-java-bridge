use crate::node::config::Config;
use crate::node::java_class_instance::EMPTY_STRING;

#[napi]
pub struct JavaConfig;

#[napi]
impl JavaConfig {
    #[napi]
    pub fn set_run_event_loop_when_interface_proxy_is_active(value: bool) {
        Config::get().run_event_loop_when_interface_proxy_is_active = value;
    }

    #[napi]
    pub fn get_run_event_loop_when_interface_proxy_is_active() -> bool {
        Config::get().run_event_loop_when_interface_proxy_is_active
    }

    #[napi]
    pub fn set_custom_inspect(value: bool) {
        Config::get().custom_inspect = value;
    }

    #[napi]
    pub fn get_custom_inspect() -> bool {
        Config::get().custom_inspect
    }

    #[napi]
    pub fn set_sync_suffix(value: Option<String>) -> napi::Result<()> {
        if value.as_ref().unwrap_or(&EMPTY_STRING)
            == Config::get().async_suffix.as_ref().unwrap_or(&EMPTY_STRING)
        {
            Err(napi::Error::from_reason(
                "syncSuffix and asyncSuffix cannot be the same",
            ))
        } else {
            Config::get().sync_suffix = value;
            Ok(())
        }
    }

    #[napi]
    pub fn get_sync_suffix() -> Option<String> {
        Config::get().sync_suffix.clone()
    }

    #[napi]
    pub fn set_async_suffix(value: Option<String>) -> napi::Result<()> {
        if value.as_ref().unwrap_or(&EMPTY_STRING)
            == Config::get().sync_suffix.as_ref().unwrap_or(&EMPTY_STRING)
        {
            Err(napi::Error::from_reason(
                "syncSuffix and asyncSuffix cannot be the same",
            ))
        } else {
            Config::get().async_suffix = value;
            Ok(())
        }
    }

    #[napi]
    pub fn get_async_suffix() -> Option<String> {
        Config::get().async_suffix.clone()
    }

    #[napi]
    pub fn set(value: Config) -> napi::Result<()> {
        if value.async_suffix.as_ref().unwrap_or(&EMPTY_STRING)
            == value.sync_suffix.as_ref().unwrap_or(&EMPTY_STRING)
        {
            Err(napi::Error::from_reason(
                "syncSuffix and asyncSuffix cannot be the same",
            ))
        } else {
            Config::get().clone_from(&value);
            Ok(())
        }
    }

    #[napi]
    pub fn get() -> Config {
        Config::get().clone()
    }

    #[napi]
    pub fn reset() {
        Config::get().clone_from(&Config::default());
    }
}
