#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "log")]
        log::debug!($($arg)*);
    };
}

/*#[cfg(not(feature = "log"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {};
}*/
