use std::cell::UnsafeCell;
use std::fmt::{Debug, Display};
use std::sync::OnceLock;

#[macro_use]
pub mod macros;
mod render;

pub use ftrace_macros::*;
use owo_colors::{OwoColorize, Style, Styled};

use crate::render::{RenderContext, Renderable};

#[derive(Default)]
pub struct Subscriber {
    depth: usize,
}

unsafe impl Send for Subscriber {}
unsafe impl Sync for Subscriber {}

impl Subscriber {
    #[must_use = "This function returns a guard object to exit the span.
        Dropping it immediately is probably incorrect. Make sure that the returned value
        lives until the span is exited."]
    pub fn enter_span(&mut self, metadata: SpanMetadata) -> SpanGuard {
        let cx = RenderContext {
            depth: self.depth,
            level: metadata.level,
        };

        let mut stdout = std::io::stdout();
        metadata.render_to(&cx, &mut stdout).unwrap();

        self.depth += 1;

        SpanGuard
    }

    pub fn event(&self, metadata: EventMetadata) {
        let cx = RenderContext {
            depth: self.depth,
            level: metadata.level,
        };

        let mut stdout = std::io::stdout();
        metadata.render_to(&cx, &mut stdout).unwrap();
    }

    pub fn exit_span(&mut self, _span: &SpanGuard) {
        self.depth -= 1;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
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

    pub fn with_field(mut self, key: &'static str, value: impl Debug + 'static) -> Self {
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

    pub fn with_field(mut self, key: &'static str, value: impl Debug + 'static) -> Self {
        self.fields.add(key, value);
        self
    }
}

#[derive(Default)]
struct FieldSet {
    inner: Vec<(&'static str, Value)>,
}

impl FieldSet {
    pub fn add(&mut self, key: &'static str, value: impl Debug + 'static) {
        self.inner.push((key, Value(Box::new(value))));
    }
}

pub struct Value(Box<dyn Debug>);

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.0.as_ref(), f)
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
