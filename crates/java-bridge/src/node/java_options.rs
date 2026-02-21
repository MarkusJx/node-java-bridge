/// Options for the Java VM.
/// Not the same as vm arguments.
#[napi(object)]
#[derive(Default)]
pub struct JavaOptions {
    /// Additional items to add to the class path. This does allow for wildcard imports
    /// using glob patterns. If a path is unreadable, an error will be thrown.
    /// This behavior can be changed by setting `ignore_unreadable_class_path_entries` to true.
    pub classpath: Option<Vec<String>>,
    /// Whether to ignore unreadable class path entries
    pub ignore_unreadable_class_path_entries: Option<bool>,
}
