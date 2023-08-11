use crate::node::config::Config;

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
}
