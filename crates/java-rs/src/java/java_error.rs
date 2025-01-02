use crate::objects::object::GlobalJavaObject;
use std::error::Error;
use std::fmt::Debug;

#[derive(Debug)]
pub struct JavaError {
    causes: Vec<String>,
    stack_frames: Vec<String>,
    alt_text: String,
    throwable: Option<GlobalJavaObject>,
}

impl JavaError {
    pub fn new(causes: Vec<String>, stack_frames: Vec<String>, alt_text: String) -> Self {
        Self {
            causes,
            stack_frames,
            alt_text,
            throwable: None,
        }
    }

    pub fn new_with_throwable(
        causes: Vec<String>,
        stack_frames: Vec<String>,
        alt_text: String,
        throwable: GlobalJavaObject,
    ) -> Self {
        let res = Self {
            causes,
            stack_frames,
            alt_text,
            throwable: Some(throwable),
        };

        #[cfg(feature = "log")]
        log::debug!("Exception thrown:\n{res}");

        res
    }

    pub fn get_throwable(&self) -> Option<GlobalJavaObject> {
        self.throwable.clone()
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

impl Error for JavaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}
