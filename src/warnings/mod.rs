//! Warnings of the wasm plugin.
//!
//! swc drops the warnings of a transform that succeeds, so the plugin prints them itself, in the
//! format of babel-plugin-inferno: `file:line:column: message` and a code frame like the one of
//! `@babel/code-frame`.

use std::sync::{Arc, Mutex};
use swc_core::common::{
    SourceMapper, Span,
    errors::{Diagnostic, DiagnosticBuilder, Emitter, HANDLER, Handler, Level},
};

#[cfg(test)]
mod tests;

const PREFIX: &str = "swc-plugin-inferno: ";

/// Keeps the diagnostics reported to its handler
#[derive(Clone, Default)]
struct Collector(Arc<Mutex<Vec<Diagnostic>>>);

impl Emitter for Collector {
    fn emit(&mut self, db: &mut DiagnosticBuilder<'_>) {
        self.0.lock().unwrap().push((**db).clone());
    }
}

/// Runs `op` and returns the warnings it reports, formatted for printing. The other diagnostics
/// are passed on to the current handler.
pub(crate) fn collect_warnings<S, R>(
    cm: &S,
    filename: &str,
    op: impl FnOnce() -> R,
) -> (R, Vec<String>)
where
    S: SourceMapper,
{
    let collector = Collector::default();
    let handler = Handler::with_emitter(true, false, Box::new(collector.clone()));
    let result = HANDLER.set(&handler, op);
    let mut warnings = vec![];

    for diagnostic in collector.0.lock().unwrap().drain(..) {
        if diagnostic.level == Level::Warning {
            warnings.push(format_warning(
                cm,
                filename,
                diagnostic.span.primary_span(),
                &diagnostic.message(),
            ));
        } else {
            HANDLER.with(|handler| DiagnosticBuilder::new_diagnostic(handler, diagnostic).emit());
        }
    }

    (result, warnings)
}

/// `reportUselessFlag` of babel-plugin-inferno
fn format_warning<S>(cm: &S, filename: &str, span: Option<Span>, message: &str) -> String
where
    S: SourceMapper,
{
    // JSX built by other plugins has no position, and no code to show
    let Some(span) = span.filter(|span| !span.is_dummy()) else {
        return format!("{PREFIX}{filename}: {message}");
    };
    let file = cm.lookup_char_pos(span.lo).file;
    let lines = split_lines(&file.src);
    let start = position(&lines, (span.lo - file.start_pos).0 as usize);
    let end = position(&lines, (span.hi - file.start_pos).0 as usize);

    format!(
        "{PREFIX}{filename}:{}:{}: {message}\n{}",
        start.line,
        start.column + 1,
        code_frame(&lines, start, end)
    )
}

/// A position like the ones of Babel: lines from 1, columns from 0 in UTF-16 code units
#[derive(Clone, Copy)]
struct Position {
    line: usize,
    column: usize,
}

/// The lines of `src` with their byte offsets, split at the line terminators of JavaScript like
/// `@babel/code-frame` splits them
fn split_lines(src: &str) -> Vec<(usize, &str)> {
    let mut lines = vec![];
    let mut start = 0;
    let mut chars = src.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        let len = match c {
            '\r' if chars.next_if(|&(_, next)| next == '\n').is_some() => 2,
            '\n' | '\r' | '\u{2028}' | '\u{2029}' => c.len_utf8(),
            _ => continue,
        };
        lines.push((start, &src[start..i]));
        start = i + len;
    }
    lines.push((start, &src[start..]));

    lines
}

fn position(lines: &[(usize, &str)], offset: usize) -> Position {
    let index = lines.partition_point(|&(start, _)| start <= offset) - 1;
    let (start, text) = lines[index];
    let before = text.get(..offset - start).unwrap_or(text);

    Position {
        line: index + 1,
        column: before.encode_utf16().count(),
    }
}

/// `codeFrameColumns` of `@babel/code-frame`, without colors: two lines above the position and
/// three below it
fn code_frame(lines: &[(usize, &str)], start: Position, end: Position) -> String {
    // Babel passes no end position when the node ends on another line, which marks one character
    let markers = if end.line == start.line {
        end.column - start.column
    } else {
        0
    }
    .max(1);
    let first = start.line.saturating_sub(3) + 1;
    let last = lines.len().min(start.line + 3);
    let width = last.to_string().len();
    let mut frame = vec![];

    for number in first..=last {
        let line = lines[number - 1].1;
        let code = if line.is_empty() {
            String::new()
        } else {
            format!(" {line}")
        };

        if number == start.line {
            // Tabs are kept so that the markers line up with the code
            let spacing: String = line
                .chars()
                .scan(0, |column, c| {
                    *column += c.len_utf16();
                    (*column <= start.column).then_some((c, c.len_utf16()))
                })
                .flat_map(|(c, len)| std::iter::repeat_n(if c == '\t' { '\t' } else { ' ' }, len))
                .collect();

            frame.push(format!(
                "> {number:>width$} |{code}\n  {:width$} | {spacing}{}",
                "",
                "^".repeat(markers)
            ));
        } else {
            frame.push(format!("  {number:>width$} |{code}"));
        }
    }

    frame.join("\n")
}
