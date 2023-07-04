#[derive(Debug)]
pub struct JNIError(String);

impl JNIError {
    pub fn new(msg: String) -> Self {
        Self(msg)
    }
}

impl From<&str> for JNIError {
    fn from(msg: &str) -> Self {
        Self(msg.to_string())
    }
}

impl From<String> for JNIError {
    fn from(msg: String) -> Self {
        Self(msg)
    }
}

impl std::fmt::Display for JNIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for JNIError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}
