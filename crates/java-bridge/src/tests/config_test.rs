use crate::node::config::Config;

#[test]
pub fn test_default() {
    let config = Config::default();
    assert!(!config.run_event_loop_when_interface_proxy_is_active);
    assert!(!config.custom_inspect);
}
