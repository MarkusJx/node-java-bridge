use std::sync::{Mutex, MutexGuard};

use lazy_static::lazy_static;

lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new(Config::default());
}

pub struct Config {
    pub run_event_loop_when_interface_proxy_is_active: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            run_event_loop_when_interface_proxy_is_active: false,
        }
    }
}

impl Config {
    pub fn get<'a>() -> MutexGuard<'a, Config> {
        CONFIG.lock().unwrap()
    }
}
