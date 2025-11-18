/// Creates a new event in the current span.
///
/// The event macro is invoked with a [`crate::Level`], along with a message.
/// The message may be a format string, followed by zero-or-more arguments.
#[macro_export]
macro_rules! event {
    ($($args:tt)*) => {};
}

/// Creates a new trace-level event in the current span.
///
/// This macro functions similarly to the [`event!`][event] macro. See [the
/// top-level documentation][crate] for details.
///
/// [event]: crate::event!
/// [crate]: crate#macros
#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => {};
}

/// Creates a new debug-level event in the current span.
///
/// This macro functions similarly to the [`event!`][event] macro. See [the
/// top-level documentation][crate] for details.
///
/// [event]: crate::event!
/// [crate]: crate#macros
#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => {};
}

/// Creates a new info-level event in the current span.
///
/// This macro functions similarly to the [`event!`][event] macro. See [the
/// top-level documentation][crate] for details.
///
/// [event]: crate::event!
/// [crate]: crate#macros
#[macro_export]
macro_rules! info {
    ($($args:tt)*) => {};
}

/// Creates a new warning-level event in the current span.
///
/// This macro functions similarly to the [`event!`][event] macro. See [the
/// top-level documentation][crate] for details.
///
/// [event]: crate::event!
/// [crate]: crate#macros
#[macro_export]
macro_rules! warning {
    ($($args:tt)*) => {};
}

/// Creates a new error-level event in the current span.
///
/// This macro functions similarly to the [`event!`][event] macro. See [the
/// top-level documentation][crate] for details.
///
/// [event]: crate::event!
/// [crate]: crate#macros
#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {};
}
