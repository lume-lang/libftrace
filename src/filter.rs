use crate::{EventMetadata, FieldSet, Level, SpanMetadata};

/// A filter for filtering out unwanted spans and events, based on a set of
/// directives.
///
/// The [`EnvFilter`] is meant to be created for debug purposes, most useful
/// when trying to track down a specific error. `EnvFilter`s are usually created
/// via the [`from_env`] function.
///
/// # Directives
///
/// A [`EnvFilter`] is effectively a list of zero-or-more directives, which
/// determines which spans and events should be emitted to the user. Directives
/// are comma-separated and each take a single verbosity level (in the form of a
/// [Level]), which defines the maximum verbosity to allow through.
///
/// Each directive has multiple different parts, which determine which items it
/// targets:
/// ```ignore
/// target[field=value]=level
/// ```
///
/// - `target` matches the name of the function or method which the span or
///   event was emitted from. `target` only matches the first part of the target
///   name - of the `target` filter is set to `backend`, spans and events from
///   any nested functions, such as `backend::api` and `backend::db` are also
///   matched.
///
/// - `field` is used to match fields within a span or event. Each field has a
///   corresponding "mode" and "value". Modes define how the field value should
///   be checked - currently there are 4 modes:
///     - `=`: field value **must equal** with the given filter value
///     - `~=`: field value **must contain** with the given filter value
///     - `^=`: field value **must start** with the given filter value
///     - `$=`: field value **must end** with the given filter value
///
///   Following the field mode, `value`s match the value of the field itself,
///   depending on the mode. For example:
///     - `[name~="John"]`: matches all items which have a field, `name`, which
///       contains the value `John`.
///     - `[description^="Fantastic"]`: matches all items which have a field,
///       `description`, which start with the value `Fantastic`.
///
/// - `level` defines the maximum level of the directive. If any span or event
///   matches the directive, it must also have a verbosity level which is equal
///   or less than this level.
#[derive(Debug)]
pub struct EnvFilter {
    directives: Vec<Directive>,
    default_level: Option<Level>,
}

impl EnvFilter {
    /// Creates a new [`EnvFilter`] from the given list of [`Directive`]s.
    fn from_directives(mut directives: Vec<Directive>) -> Self {
        let default_level = directives
            .iter()
            .position(|d| d.module.is_none() && d.fields.is_empty())
            .map(|idx| directives.remove(idx).level);

        EnvFilter {
            directives,
            default_level,
        }
    }
}

impl Default for EnvFilter {
    fn default() -> Self {
        Self::from_directives(vec![Directive {
            module: None,
            fields: Vec::new(),
            level: Level::Error,
        }])
    }
}

#[derive(Debug)]
struct Directive {
    pub module: Option<String>,
    pub fields: Vec<FieldFilter>,
    pub level: Level,
}

#[derive(Debug)]
struct FieldFilter {
    pub key: String,
    pub value: String,
    pub mode: FilterMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterMode {
    Equal,
    Contains,
    StartsWith,
    EndsWith,
}

/// Defines the default environment variable to use in [`from_default_env`].
pub const DEFAULT_ENV: &str = "RUST_LOG";

/// Reads the value of the `RUST_LOG` environment variable and parses it into
/// an [`EnvFilter`], returning any raised errors to the caller.
///
/// If the environment variable is empty or unset, a default [`EnvFilter`] is
/// returned with a single `error` directive.
///
/// If you want to defined which environment variable to read from, see
/// [`from_env`].
pub fn from_default_env() -> Result<EnvFilter, ParseError> {
    from_env(DEFAULT_ENV)
}

/// Reads the value of the given environment variable and parses it into
/// an [`EnvFilter`], returning any raised errors to the caller.
///
/// If the environment variable is empty or unset, a default [`EnvFilter`] is
/// returned with a single `error` directive.
pub fn from_env(env_name: &str) -> Result<EnvFilter, ParseError> {
    let Some(env_value) = std::env::var_os(env_name) else {
        return Ok(EnvFilter::default());
    };

    if env_value.is_empty() {
        return Ok(EnvFilter::default());
    }

    parse(env_value.to_string_lossy())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// A directive was defined without having any level.
    MissingLevel,

    /// A level was given to a directive, but was invalid or malformed.
    InvalidLevel(String),
}

/// Parses the given value into an [`EnvFilter`], returning any raised errors to
/// the caller.
pub fn parse<V: AsRef<str>>(from: V) -> Result<EnvFilter, ParseError> {
    let mut directives = Vec::new();
    let mut parser = Parser {
        slice: from.as_ref(),
        idx: 0,
    };

    while !parser.eof() {
        directives.push(parser.parse_directive()?);

        // Unless there's a delimiting comma, stop parsing.
        if !parser.check(',') {
            break;
        }
    }

    Ok(EnvFilter::from_directives(directives))
}

struct Parser<'src> {
    slice: &'src str,
    idx: usize,
}

impl<'src> Parser<'src> {
    #[inline]
    pub fn eof(&self) -> bool {
        self.idx >= self.slice.len() - 1
    }

    #[inline]
    fn peek(&self) -> Option<char> {
        self.slice.chars().nth(self.idx)
    }

    #[inline]
    fn check(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.idx += 1;
            return true;
        }

        false
    }

    #[inline]
    fn take_while<F: FnMut(char) -> bool>(&mut self, mut f: F) -> Option<&'src str> {
        let start = self.idx;
        let mut ci = self.slice[self.idx..].chars().peekable();

        while let Some(c) = ci.peek()
            && f(*c)
        {
            self.idx += 1;
            ci.next();
        }

        if start == self.idx {
            return None;
        }

        Some(&self.slice[start..self.idx])
    }

    #[inline]
    fn identifier(&mut self) -> Option<&'src str> {
        self.take_while(|c| c.is_ascii_alphabetic())
    }

    #[inline]
    fn module_name(&mut self) -> Option<&'src str> {
        self.take_while(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_'))
    }

    #[inline]
    fn value(&mut self) -> Option<&'src str> {
        if self.peek() == Some('"') {
            let start = self.idx;
            let mut ci = self.slice[self.idx..].char_indices().peekable();

            match ci.next() {
                Some((offset, '"')) => return Some(&self.slice[start..start + offset]),
                None => return None,
                _ => {}
            }
        }

        self.take_while(|c| c.is_ascii_alphanumeric())
    }

    pub fn parse_directive(&mut self) -> Result<Directive, ParseError> {
        let mut directive = Directive {
            module: None,
            fields: Vec::new(),
            level: Level::Info,
        };

        if let Some(module_str) = self.module_name() {
            directive.module = Some(module_str.to_string());
        }

        // Parse zero-or-more field filters in the directive.
        if self.check('[') {
            while !self.check(']') {
                directive.fields.push(self.parse_filter()?);

                // Unless there's a delimiting comma, stop parsing.
                if !self.check(',') {
                    break;
                }
            }

            self.check(']');
        }

        // Parse the level required for any items to pass through it.
        if self.check('=') {
            let Some(level_str) = self.identifier() else {
                return Err(ParseError::MissingLevel);
            };

            directive.level = match Level::try_from(level_str) {
                Ok(level) => level,
                Err(_) => return Err(ParseError::InvalidLevel(level_str.to_string())),
            };
        }
        // If we don't see an assignment for the level, we assume that the entire directive
        // is a single level filter, such as:
        // ```
        // RUST_LOG=info
        // ```
        else if let Some(module_str) = directive.module.take() {
            directive.level = match Level::try_from(module_str.as_str()) {
                Ok(level) => level,
                Err(_) => return Err(ParseError::InvalidLevel(module_str)),
            };
        }

        Ok(directive)
    }

    fn parse_filter(&mut self) -> Result<FieldFilter, ParseError> {
        let mut filter = FieldFilter {
            key: String::new(),
            mode: FilterMode::Equal,
            value: String::new(),
        };

        // Parse the name of the field filter.
        let Some(key_str) = self.identifier() else { todo!() };
        filter.key = key_str.to_string();

        // Parse the mode of the field filter.
        filter.mode = if self.check('=') {
            FilterMode::Equal
        } else if self.check('~') && self.check('=') {
            FilterMode::Contains
        } else if self.check('^') && self.check('=') {
            FilterMode::StartsWith
        } else if self.check('$') && self.check('=') {
            FilterMode::EndsWith
        } else {
            todo!()
        };

        // Parse the matching value of the field filter.
        filter.value = if let Some(value) = self.value() {
            value.to_string()
        } else {
            todo!()
        };

        Ok(filter)
    }
}

impl EnvFilter {
    /// Attempts to determine whether the given [`SpanMetadata`] should be
    /// emitted, given the current directives of the filter.
    pub fn span_enabled(&self, span: &SpanMetadata) -> bool {
        let directives: Vec<&Directive> = self.directives_for_span(span).collect();

        if directives.is_empty() {
            if let Some(default_level) = self.default_level {
                return default_level <= span.level;
            }

            // If there's no applicable directives and no default level,
            // the span should not be emitted.
            return false;
        }

        // If any of the directive filters are met, the span should be emitted.
        for directive in directives {
            if span.level >= directive.level {
                return true;
            }
        }

        false
    }

    /// Attempts to determine whether the given [`EventMetadata`] should be
    /// emitted, given the current directives of the filter.
    pub fn event_enabled(&self, event: &EventMetadata, parent_span: Option<&SpanMetadata>) -> bool {
        let directives: Vec<&Directive> = self.directives_for_event(event, parent_span).collect();

        if directives.is_empty() {
            if let Some(default_level) = self.default_level {
                return default_level <= event.level;
            }

            // If there's no applicable directives and no default level,
            // the event should not be emitted.
            return false;
        }

        // If any of the directive filters are met, the event should be emitted.
        for directive in directives {
            if event.level >= directive.level {
                return true;
            }
        }

        false
    }

    /// Returns an iterator of all the directives which would handle the given
    /// [`SpanMetadata`].
    fn directives_for_span(&self, span: &SpanMetadata) -> impl Iterator<Item = &Directive> {
        self.directives.iter().filter(|dir| dir.handles_span(span))
    }

    /// Returns an iterator of all the directives which would handle the given
    /// [`EventMetadata`].
    fn directives_for_event(
        &self,
        event: &EventMetadata,
        parent_span: Option<&SpanMetadata>,
    ) -> impl Iterator<Item = &Directive> {
        self.directives.iter().filter(move |dir| {
            parent_span.is_some_and(|span| dir.handles_span(span)) && dir.handles_field_set(&event.fields)
        })
    }
}

impl Directive {
    /// Determines whether the current [`Directive`] would handle the given
    /// [`SpanMetadata`].
    fn handles_span(&self, span: &SpanMetadata) -> bool {
        if self.module.as_ref().is_some_and(|m| !span.name.starts_with(m)) {
            return false;
        }

        self.handles_field_set(&span.fields)
    }

    /// Determines whether the current [`Directive`] would handle the given
    /// [`FieldSet`].
    fn handles_field_set(&self, field_set: &FieldSet) -> bool {
        for filter in &self.fields {
            let Some((_, field)) = field_set.inner.iter().find(|(k, _)| *k == filter.key) else {
                return false;
            };

            let field_value = format!("{field}");
            let field_value = field_value.trim_matches('"');

            match filter.mode {
                FilterMode::Equal => {
                    if field_value != filter.value {
                        return false;
                    }
                }
                FilterMode::Contains => {
                    if !field_value.contains(&filter.value) {
                        return false;
                    }
                }
                FilterMode::StartsWith => {
                    if !field_value.starts_with(&filter.value) {
                        return false;
                    }
                }
                FilterMode::EndsWith => {
                    if !field_value.ends_with(&filter.value) {
                        return false;
                    }
                }
            }
        }

        true
    }
}
