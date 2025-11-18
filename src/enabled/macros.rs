/// Creates a new event in the current span.
///
/// The event macro is invoked with a [`crate::Level`], along with a message.
/// The message may be a format string, followed by zero-or-more arguments.
#[macro_export]
macro_rules! event {
    (level: $level:expr, $fmt:expr, $( $key:ident = $value:expr ),+) => {
        $crate::with_subscriber(|s| {
            s.event(
                $crate::EventMetadata::new(format!($fmt), $level)
                $(
                    .with_field(stringify!($key), $value)
                )*
            );
        });
    };
    (level: $level:expr, $fmt:expr, $($args:expr)*, $( $key:ident = $value:expr ),+) => {
        $crate::with_subscriber(|s| {
            s.event(
                $crate::EventMetadata::new(format!($fmt, $($args)*), $level)
                $(
                    .with_field(stringify!($key), $value)
                )*
            );
        });
    };
    (level: $level:expr, $fmt:expr) => {
        $crate::with_subscriber(|s| {
            s.event($crate::EventMetadata::new(format!($fmt), $level));
        });
    };
    (level: $level:expr, $fmt:expr, $($args:tt)*) => {
        $crate::with_subscriber(|s| {
            s.event($crate::EventMetadata::new(format!($fmt, $($args)*), $level));
        });
    };
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
    ($($args:tt)*) => {
        $crate::event!(level: $crate::Level::Trace, $($args)*);
    };
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
    ($($args:tt)*) => {
        $crate::event!(level: $crate::Level::Debug, $($args)*);
    };
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
    ($($args:tt)*) => {
        $crate::event!(level: $crate::Level::Info, $($args)*);
    };
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
    ($($args:tt)*) => {
        $crate::event!(level: $crate::Level::Warn, $($args)*);
    };
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
    ($($args:tt)*) => {
        $crate::event!(level: $crate::Level::Error, $($args)*);
    };
}
