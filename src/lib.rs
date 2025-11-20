//! Extemely simple function tracer, useful for debugging.
//!
//! # Overview
//!
//! `libftrace` is a very simple library, focused on giving a complete overview
//! into the execution path of Rust programs, by emitting spans and events.
//!
//! Similar to the much larger [`tracing`] crate, `libftrace` has *spans* and
//! *events*.
//!
//! [`tracing`]: https://docs.rs/tracing/latest/tracing/
//!
//! ## Usage
//!
//! Before diving too deep, you should add the crate to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! libftrace = "^0"
//! ```
//!
//! Or use the following command to add it:
//! ```sh
//! cargo add libftrace
//! ```
//!
//! ### Spans
//!
//! Spans are meant to represent a flow of execution through a program, spanning
//! over some period of time. Each span can have zero-or-more child spans,
//! representing sub-tasks within a larger span.
//!
//! When thinking of spans in common Rust programs, it's most common to mirror
//! them up against the execution of functions, since they follow very similar
//! logic: much like a span, a function starts execution at some point in time,
//! perhaps executes other functions in it's body, and stops execution at some
//! later point in time.
//!
//! In `libftrace`, function execution is thought to be analogous to spans, so
//! spans are very simple to attach to a function. But, we'll talk more about
//! that later.
//!
//! ### Events
//!
//! Unlike spans which span over a period of time, events represent a single
//! moment in time, often inside a parent span.
//!
//! Events are used to signify something happening during a span. This can be
//! an error occuring, a note-worthy change within the program or other
//! information which can be useful.
//!
//! ### Macros
//!
//! #### Spans
//!
//! To mark a function as a spanning execution, you can add the `#[traced]`
//! attribute. This provides a very easy, yet powerful, way of marking execution
//! paths within your programs lifecycle.
//!
//! [`#[traced]`]: ftrace_macros::traced
//!
//! For example:
//! ```rs
//! #[traced]
//! fn handle_request(req: Request) {
//!     // ..
//! }
//! ```
//!
//! By default, `#[traced]` with use the [`Info`][`Level::Info`] verbosity
//! level, if nothing else is defined. To change this, add the `level` argument
//! to the attribute:
//! ```
//! use libftrace::*;
//!
//! #[traced(level = Debug)]
//! pub fn my_function() {
//!     // ...
//! }
//! ```
//!
//! #### Events
//!
//! Events can be created using the [`event!`] macro. It allows for a very
//! quick way to emit some event to the subscriber:
//! ```
//! use libftrace::*;
//!
//! event!(level: Level::Info, "a new user logged in!");
//! ```
//!
//! [`event!`]: crate::event!
//!
//! For convinience, there are also macros for each log level:
//! ```
//! use libftrace::*;
//!
//! trace!("event sent to backend");
//! debug!("user logged in");
//! info!("failed login attempt");
//! warning!("non-TLS request made to backend");
//! error!("product does not exist");
//! ```
//!
//! #### Fields
//!
//! Both spans and fields can have fields attached to them, allow for better
//! understanding of what is currently happening within the program. Fields
//! effectively function as a key-value map, mapping some string-based key to a
//! value.
//!
//! To attach fields with the `#[traced]` attribute, add the `fields()`
//! argument:
//! ```rs
//! #[traced(level = Info, fields(method = req.method, host = req.host))]
//! fn handle_request(req: Request) {
//!     // ..
//! }
//! ```
//!
//! For events, the syntax is very similar:
//! ```rs
//! info!("failed login attempt", username = creds.username);
//! ```

use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::fmt::Display;
use std::sync::OnceLock;

#[macro_use]
#[path = "enabled/macros.rs"]
#[cfg(feature = "enabled")]
pub mod macros;

#[macro_use]
#[path = "disabled/macros.rs"]
#[cfg(not(feature = "enabled"))]
pub mod macros;

pub mod filter;
mod render;

pub use libftrace_macros::*;
use owo_colors::{OwoColorize, Style, Styled};

pub use crate::filter::*;
use crate::render::{RenderContext, Renderable};

#[derive(Default)]
pub struct Subscriber {
    depth: usize,
    filter: Option<EnvFilter>,
    current: VecDeque<SpanMetadata>,
}

unsafe impl Send for Subscriber {}
unsafe impl Sync for Subscriber {}

impl Subscriber {
    /// Enter a new span, containing the given [`SpanMetadata`] instance.
    ///
    /// This method returns a guard for the span. When the guard is dropped,
    /// the span is exited. If this is not intended, keep the guard in scope.
    #[must_use = "This function returns a guard object to exit the span.
        Dropping it immediately is probably incorrect. Make sure that the returned value
        lives until the span is exited."]
    pub fn enter_span(&mut self, metadata: SpanMetadata) -> Option<SpanGuard> {
        if self.filter.as_ref().is_some_and(|f| !f.span_enabled(&metadata)) {
            return None;
        }

        let cx = RenderContext {
            depth: self.depth,
            level: metadata.level,
        };

        let mut stdout = std::io::stdout();
        metadata.render_to(&cx, &mut stdout).unwrap();

        self.depth += 1;
        self.current.push_front(metadata);

        Some(SpanGuard)
    }

    /// Emit the given event in the current span.
    pub fn event(&self, metadata: EventMetadata) {
        let current_span = self.current.front();

        if self
            .filter
            .as_ref()
            .is_some_and(|f| !f.event_enabled(&metadata, current_span))
        {
            return;
        }

        let cx = RenderContext {
            depth: self.depth,
            level: metadata.level,
        };

        let mut stdout = std::io::stdout();
        metadata.render_to(&cx, &mut stdout).unwrap();
    }

    pub fn exit_span(&mut self, _span: &SpanGuard) {
        self.current.pop_front();
        self.depth -= 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl TryFrom<&str> for Level {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value.to_lowercase().as_str() {
            "trace" => Ok(Level::Trace),
            "debug" => Ok(Level::Debug),
            "info" => Ok(Level::Info),
            "warn" => Ok(Level::Warn),
            "error" => Ok(Level::Error),
            _ => Err(()),
        }
    }
}

pub struct SpanMetadata {
    pub name: &'static str,
    pub location: &'static std::panic::Location<'static>,
    pub level: Level,
    fields: FieldSet,
}

impl SpanMetadata {
    #[track_caller]
    pub fn new(name: &'static str, level: Level) -> Self {
        Self {
            name,
            level,
            location: std::panic::Location::caller(),
            fields: FieldSet::default(),
        }
    }

    pub fn with_field(mut self, key: &'static str, value: impl Display + 'static) -> Self {
        self.fields.add(key, value);
        self
    }
}

pub struct EventMetadata {
    pub message: String,
    pub location: &'static std::panic::Location<'static>,
    pub level: Level,
    fields: FieldSet,
}

impl EventMetadata {
    #[track_caller]
    pub fn new<S: Into<String>>(message: S, level: Level) -> Self {
        Self {
            message: message.into(),
            level,
            location: std::panic::Location::caller(),
            fields: FieldSet::default(),
        }
    }

    pub fn with_field(mut self, key: &'static str, value: impl Display + 'static) -> Self {
        self.fields.add(key, value);
        self
    }
}

#[derive(Default)]
struct FieldSet {
    inner: Vec<(&'static str, Value)>,
}

impl FieldSet {
    pub fn add(&mut self, key: &'static str, value: impl Display + 'static) {
        self.inner.push((key, Value(Box::new(value))));
    }
}

pub struct Value(Box<dyn Display>);

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.0.as_ref(), f)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SpanGuard;

impl Drop for SpanGuard {
    fn drop(&mut self) {
        with_subscriber(|subscriber| subscriber.exit_span(self))
    }
}

struct Global<T> {
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for Global<T> where T: Send {}

static GLOBAL: OnceLock<Global<Subscriber>> = OnceLock::new();

pub fn with_subscriber<F: FnOnce(&mut Subscriber) -> R, R>(f: F) -> R {
    let global = GLOBAL.get_or_init(|| Global {
        inner: UnsafeCell::new(Subscriber::default()),
    });

    unsafe { f(&mut *global.inner.get()) }
}

/// Sets the current filter of the global trace subscriber.
///
/// To create a [`EnvFilter`] instance, see [`from_env`], [`from_default_env`]
/// or [`parse`].
pub fn set_filter(filter: EnvFilter) {
    with_subscriber(|subscriber| subscriber.filter = Some(filter));
}
