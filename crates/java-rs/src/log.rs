#[cfg(feature = "log")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        log::error!($($arg)*);
    };
}

#[cfg(not(feature = "log"))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {};
}

#[cfg(feature = "log")]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        log::trace!($($arg)*);
    };
}

#[cfg(not(feature = "log"))]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {};
}
