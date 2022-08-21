pub mod array;
pub mod class;
pub mod constructor;
pub mod java_object;
pub mod method;
pub mod object;
pub mod string;
pub mod value;

pub mod args {
    use crate::jni::traits::ToJavaValue;

    pub type JavaArg<'a> = Box<&'a dyn ToJavaValue<'a>>;
    pub type JavaArgs<'a> = Vec<JavaArg<'a>>;
}
