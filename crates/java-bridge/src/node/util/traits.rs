use lazy_static::lazy_static;

lazy_static! {
    static ref EMPTY_STRING: String = "".to_string();
}

pub trait UnwrapOrEmpty {
    fn unwrap_or_empty(&self) -> &String;
}

impl UnwrapOrEmpty for Option<&String> {
    fn unwrap_or_empty(&self) -> &String {
        self.unwrap_or(&EMPTY_STRING)
    }
}

impl UnwrapOrEmpty for Option<String> {
    fn unwrap_or_empty(&self) -> &String {
        self.as_ref().unwrap_or(&EMPTY_STRING)
    }
}
