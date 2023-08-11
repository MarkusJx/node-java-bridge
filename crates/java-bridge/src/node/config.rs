use std::sync::{Mutex, MutexGuard};

use lazy_static::lazy_static;
use smart_default::SmartDefault;

lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new(Config::default());
}

#[derive(SmartDefault)]
pub struct Config {
    #[default(false)]
    pub run_event_loop_when_interface_proxy_is_active: bool,
    #[default(false)]
    pub custom_inspect: bool,
}

impl Config {
    pub fn get<'a>() -> MutexGuard<'a, Config> {
        CONFIG.lock().unwrap()
    }
}
