#[derive(Debug)]
pub struct JavaError {
    causes: Vec<String>,
    stack_frames: Vec<String>,
    alt_text: String,
}

impl JavaError {
    pub fn new(causes: Vec<String>, stack_frames: Vec<String>, alt_text: String) -> Self {
        Self {
            causes,
            stack_frames,
            alt_text,
        }
    }
}

impl std::fmt::Display for JavaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut causes = self.causes.clone();
        let root = causes.pop();

        let new_line = if !self.stack_frames.is_empty() {
            "\n"
        } else {
            ""
        };

        let stack_frames = self
            .stack_frames
            .clone()
            .into_iter()
            .map(|f| format!("    at {}", f))
            .collect::<Vec<_>>()
            .join("\n");

        if root.is_some() {
            write!(f, "{}{}{}", root.unwrap(), new_line, stack_frames)
        } else {
            write!(f, "{}{}{}", self.alt_text, new_line, stack_frames)
        }
    }
}

impl std::error::Error for JavaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}
