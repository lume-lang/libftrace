use std::io::Write;

use crate::*;

pub(crate) trait Renderable {
    fn render_to(&self, cx: &RenderContext, f: &mut dyn Write) -> std::io::Result<()>;
}

#[derive(Clone, Copy)]
pub(crate) struct RenderContext {
    pub depth: usize,
    pub level: Level,
}

impl RenderContext {
    #[inline]
    fn write_ident(&self, f: &mut dyn Write) -> std::io::Result<()> {
        write!(f, "{:<width$}", "", width = self.depth * 2)
    }

    #[inline]
    fn write_gutter(&self, f: &mut dyn Write) -> std::io::Result<()> {
        self.write_ident(f)?;

        write!(f, "    ")
    }
}

impl Renderable for SpanMetadata {
    fn render_to(&self, cx: &RenderContext, f: &mut dyn Write) -> std::io::Result<()> {
        cx.write_ident(f)?;

        time::UtcDateTime::now().render_to(cx, f)?;
        write!(f, " ")?;

        self.level.render_to(cx, f)?;
        writeln!(f, "  {}", self.name)?;

        self.fields.render_to(cx, f)?;
        self.location.render_to(cx, f)?;

        writeln!(f)?;
        writeln!(f)?;

        Ok(())
    }
}

impl Renderable for EventMetadata {
    fn render_to(&self, cx: &RenderContext, f: &mut dyn Write) -> std::io::Result<()> {
        cx.write_ident(f)?;

        time::UtcDateTime::now().render_to(cx, f)?;
        write!(f, " ")?;

        self.level.render_to(cx, f)?;
        writeln!(f, "  {}", self.message)?;

        self.fields.render_to(cx, f)?;
        self.location.render_to(cx, f)?;

        writeln!(f)?;
        writeln!(f)?;

        Ok(())
    }
}

impl Renderable for FieldSet {
    fn render_to(&self, cx: &RenderContext, f: &mut dyn Write) -> std::io::Result<()> {
        let field_len = self.inner.len();

        if self.inner.is_empty() {
            return Ok(());
        }

        cx.write_gutter(f)?;
        write!(f, "{} ", "with".dimmed())?;

        for (idx, (key, value)) in self.inner.iter().enumerate() {
            print!("{}", with_level_styling(cx.level, format!("{key}: {value}")));

            if idx < field_len - 1 {
                print!("{}", ", ".dimmed());
            }
        }

        writeln!(f)
    }
}

impl Renderable for Level {
    fn render_to(&self, _cx: &RenderContext, f: &mut dyn Write) -> std::io::Result<()> {
        let text = match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        };

        write!(f, "{}", with_level_styling(*self, text))
    }
}

impl Renderable for time::UtcDateTime {
    fn render_to(&self, _cx: &RenderContext, f: &mut dyn Write) -> std::io::Result<()> {
        let format = time::format_description::well_known::Rfc3339;
        let display = self.format(&format).unwrap();

        write!(f, "{}", display.dimmed())
    }
}

impl Renderable for std::panic::Location<'static> {
    fn render_to(&self, cx: &RenderContext, f: &mut dyn Write) -> std::io::Result<()> {
        cx.write_gutter(f)?;

        write!(f, "{} {}:{}", "at".dimmed(), self.file(), self.line())
    }
}

pub fn with_level_styling<T>(level: Level, value: T) -> Styled<T> {
    const TRACE: Style = Style::new().cyan();
    const DEBUG: Style = Style::new().blue();
    const INFO: Style = Style::new().green();
    const WARN: Style = Style::new().yellow();
    const ERROR: Style = Style::new().red();

    match level {
        Level::Trace => TRACE.style(value),
        Level::Debug => DEBUG.style(value),
        Level::Info => INFO.style(value),
        Level::Warn => WARN.style(value),
        Level::Error => ERROR.style(value),
    }
}
