use crate::node::interface_proxy::types::JsCallResult;
use futures::channel::oneshot::Sender;
use java_rs::java_call_result::JavaCallResult;
use java_rs::util::util::ResultType;
use std::sync::Mutex;

pub struct InterfaceCall {
    pub args: Vec<JavaCallResult>,
    sender: Mutex<Option<Sender<JsCallResult>>>,
}

impl InterfaceCall {
    pub fn new(args: Vec<JavaCallResult>, sender: Sender<JsCallResult>) -> Self {
        InterfaceCall {
            args,
            sender: Mutex::new(Some(sender)),
        }
    }

    pub fn set_result(&self, result: JsCallResult) -> ResultType<()> {
        self.sender
            .lock()
            .unwrap()
            .take()
            .ok_or("The sender was already invoked".to_string())?
            .send(result)
            .map_err(|_| "Could not send result to sender".into())
    }
}
