use crate::java::traits::ToJavaValue;

pub type JavaArg<'a> = Box<&'a dyn ToJavaValue<'a>>;
pub type JavaArgs<'a> = &'a [JavaArg<'a>];

pub trait AsJavaArg<'a> {
    fn as_arg(&'a self) -> JavaArg<'a>;
}

impl<'a, T: ToJavaValue<'a>> AsJavaArg<'a> for T {
    fn as_arg(&'a self) -> JavaArg<'a> {
        Box::new(self)
    }
}
