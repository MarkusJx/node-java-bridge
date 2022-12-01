use crate::jni::java_vm::InternalJavaOptions;

/// Options for the Java VM.
/// Not the same as vm arguments.
#[napi(object)]
pub struct JavaOptions {
    /// Whether to attach new threads as daemon threads.
    pub use_daemon_threads: Option<bool>,
    /// Additional items to add to the class path. This does allow for wildcard imports
    /// using glob patterns. If a path is unreadable, an error will be thrown.
    /// This behaviour can be changed by setting `ignore_unreadable_class_path_entries` to true.
    pub classpath: Option<Vec<String>>,
    /// Whether to ignore unreadable class path entries
    pub ignore_unreadable_class_path_entries: Option<bool>,
}

impl Default for JavaOptions {
    fn default() -> Self {
        JavaOptions {
            use_daemon_threads: None,
            classpath: None,
            ignore_unreadable_class_path_entries: None,
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
