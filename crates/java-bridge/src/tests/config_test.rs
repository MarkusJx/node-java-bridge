use crate::node::config::Config;

#[test]
pub fn test_default() {
    let config = Config::default();
    assert_eq!(config.run_event_loop_when_interface_proxy_is_active, false);
    assert_eq!(config.custom_inspect, false);
}
