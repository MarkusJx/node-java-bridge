#[derive(Debug)]
pub struct LibraryError(String);

impl LibraryError {
    pub fn new(msg: &str) -> Self {
        Self(msg.to_string())
    }
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for LibraryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}
