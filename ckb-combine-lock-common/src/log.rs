#[macro_export]
macro_rules! log {
    ($fmt:literal) => {
        #[cfg(features = "log")]
        $crate::syscalls::debug(alloc::format!($fmt));
    };
    ($fmt:literal, $($args:expr),+) => {
        #[cfg(features = "log")]
        $crate::syscalls::debug(alloc::format!($fmt, $($args), +));
        // Avoid unused warnings.
        #[cfg(not(features = "log"))]
        core::mem::drop(($(&$args),+));
    };
}
