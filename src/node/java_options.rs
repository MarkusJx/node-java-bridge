use crate::jni::java_vm::InternalJavaOptions;

/// Options for the Java VM.
/// Not the same as vm arguments.
#[napi(object)]
pub struct JavaOptions {
    /// Whether to attach new threads as daemon threads.
    pub use_daemon_threads: Option<bool>,
}

impl Default for JavaOptions {
    fn default() -> Self {
        JavaOptions {
            use_daemon_threads: None,
        }
    }
}

impl From<JavaOptions> for InternalJavaOptions {
    fn from(opts: JavaOptions) -> Self {
        InternalJavaOptions {
            use_daemon_threads: opts.use_daemon_threads.unwrap_or(false),
        }
    }
}

impl From<Option<JavaOptions>> for InternalJavaOptions {
    fn from(opts: Option<JavaOptions>) -> Self {
        Self::from(opts.unwrap_or(JavaOptions::default()))
    }
}
