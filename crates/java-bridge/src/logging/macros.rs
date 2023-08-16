#[cfg(feature = "log")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}

#[cfg(not(feature = "log"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {};
}
