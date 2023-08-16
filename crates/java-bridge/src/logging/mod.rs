#[cfg(feature = "log")]
pub mod appender;
#[cfg(not(feature = "log"))]
mod dummies;
#[cfg(feature = "log")]
pub mod log;
pub mod macros;
#[cfg(feature = "log")]
pub mod writer;
