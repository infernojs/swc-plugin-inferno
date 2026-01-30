use super::swc_atoms::Wtf8Atom;
use swc_core::{
    atoms::wtf8::{Wtf8, Wtf8Buf},
    ecma::atoms::Atom,
    ecma::utils::str::is_line_terminator,
};

/// https://github.com/microsoft/TypeScript/blob/9e20e032effad965567d4a1e1c30d5433b0a3332/src/compiler/transformers/jsx.ts#L572-L608
///
/// JSX trims whitespace at the end and beginning of lines, except that the
/// start/end of a tag is considered a start/end of a line only if that line is
/// on the same line as the closing tag. See examples in
/// tests/cases/conformance/jsx/tsxReactEmitWhitespace.tsx
/// See also https://www.w3.org/TR/html4/struct/text.html#h-9.1 and https://www.w3.org/TR/CSS2/text.html#white-space-model
///
/// An equivalent algorithm would be:
/// - If there is only one line, return it.
/// - If there is only whitespace (but multiple lines), return `undefined`.
/// - Split the text into lines.
/// - 'trimRight' the first line, 'trimLeft' the last line, 'trim' middle lines.
/// - Decode entities on each line (individually).
/// - Remove empty lines and join the rest with " ".
#[inline]
pub(super) fn jsx_text_to_str<'a, T>(t: &'a T) -> Wtf8Atom
where
    &'a T: Into<&'a Wtf8>,
    T: ?Sized,
{
    let t = t.into();
    // Fast path: JSX text is almost always valid UTF-8
    if let Some(s) = t.as_str() {
        return jsx_text_to_str_impl(s).into();
    }

    // Slow path: Handle Wtf8 with surrogates (extremely rare)
    jsx_text_to_str_wtf8_impl(t)
}

/// Handle JSX text with surrogates
fn jsx_text_to_str_wtf8_impl(t: &Wtf8) -> Wtf8Atom {
    let mut acc: Option<Wtf8Buf> = None;
    let mut only_line: Option<(usize, usize)> = None; // (start, end) byte positions
    let mut first_non_whitespace: Option<usize> = Some(0);
    let mut last_non_whitespace: Option<usize> = None;

    let mut byte_pos = 0;
    for cp in t.code_points() {
        let c = cp.to_char_lossy();
        let cp_value = cp.to_u32();

        let cp_byte_len = if cp_value < 0x80 {
            1
        } else if cp_value < 0x800 {
            2
        } else if cp_value < 0x10000 {
            3
        } else {
            4
        };

        if is_line_terminator(c) {
            if let (Some(first), Some(last)) = (first_non_whitespace, last_non_whitespace) {
                add_line_of_jsx_text_wtf8(first, last, t, &mut acc, &mut only_line);
            }
            first_non_whitespace = None;
        } else if !is_white_space_single_line(c) {
            last_non_whitespace = Some(byte_pos + cp_byte_len);
            if first_non_whitespace.is_none() {
                first_non_whitespace.replace(byte_pos);
            }
        }

        byte_pos += cp_byte_len;
    }

    if let Some(first) = first_non_whitespace {
        add_line_of_jsx_text_wtf8(first, t.len(), t, &mut acc, &mut only_line);
    }

    if let Some(acc) = acc {
        acc.into()
    } else if let Some((start, end)) = only_line {
        t.slice(start, end).into()
    } else {
        Wtf8Atom::default()
    }
}

fn add_line_of_jsx_text_wtf8(
    line_start: usize,
    line_end: usize,
    source: &Wtf8,
    acc: &mut Option<Wtf8Buf>,
    only_line: &mut Option<(usize, usize)>,
) {
    if let Some((only_start, only_end)) = only_line.take() {
        let mut buffer = Wtf8Buf::with_capacity(source.len());
        buffer.push_wtf8(source.slice(only_start, only_end));
        buffer.push_str(" ");
        buffer.push_wtf8(source.slice(line_start, line_end));
        *acc = Some(buffer);
    } else if let Some(ref mut buffer) = acc {
        buffer.push_str(" ");
        buffer.push_wtf8(source.slice(line_start, line_end));
    } else {
        *only_line = Some((line_start, line_end));
    }
}

#[inline]
fn jsx_text_to_str_impl(t: &str) -> Atom {
    let mut acc: Option<String> = None;
    let mut only_line: Option<&str> = None;
    let mut first_non_whitespace: Option<usize> = Some(0);
    let mut last_non_whitespace: Option<usize> = None;

    for (index, c) in t.char_indices() {
        if is_line_terminator(c) {
            if let (Some(first), Some(last)) = (first_non_whitespace, last_non_whitespace) {
                let line_text = &t[first..last];
                add_line_of_jsx_text(line_text, &mut acc, &mut only_line);
            }
            first_non_whitespace = None;
        } else if !is_white_space_single_line(c) {
            last_non_whitespace = Some(index + c.len_utf8());
            if first_non_whitespace.is_none() {
                first_non_whitespace.replace(index);
            }
        }
    }

    if let Some(first) = first_non_whitespace {
        let line_text = &t[first..];
        add_line_of_jsx_text(line_text, &mut acc, &mut only_line);
    }

    if let Some(acc) = acc {
        acc.into()
    } else if let Some(only_line) = only_line {
        only_line.into()
    } else {
        "".into()
    }
}

fn is_white_space_single_line(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

fn add_line_of_jsx_text<'a>(
    trimmed_line: &'a str,
    acc: &mut Option<String>,
    only_line: &mut Option<&'a str>,
) {
    if let Some(buffer) = acc.as_mut() {
        buffer.push(' ');
    } else if let Some(only_line_content) = only_line.take() {
        let mut buffer = String::with_capacity(trimmed_line.len() * 2);
        buffer.push_str(only_line_content);
        buffer.push(' ');
        *acc = Some(buffer);
    }

    if let Some(buffer) = acc.as_mut() {
        buffer.push_str(trimmed_line);
    } else {
        *only_line = Some(trimmed_line);
    }
}
