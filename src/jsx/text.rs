//! Whitespace handling of JSX text and attribute strings.
//!
//! Space, tab, `\r` and `\n` are ASCII, and WTF-8 never uses ASCII bytes inside a multi-byte
//! sequence, so the functions scan bytes and can slice the text at any of these characters.

use std::borrow::Cow;
use swc_core::atoms::{
    Wtf8Atom,
    wtf8::{CodePoint, Wtf8, Wtf8Buf},
};

fn is_blank(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

/// Appends `text` with its tabs replaced by spaces
fn push_without_tabs(buf: &mut Wtf8Buf, text: &Wtf8) {
    let mut start = 0;
    for (i, &byte) in text.as_bytes().iter().enumerate() {
        if byte == b'\t' {
            buf.push_wtf8(text.slice(start, i));
            buf.push_char(' ');
            start = i + 1;
        }
    }
    buf.push_wtf8(text.slice_from(start));
}

/// Collapses the whitespace of JSX text like `handleWhiteSpace` of
/// babel-plugin-inferno: tabs become spaces, the lines are trimmed of spaces
/// except at the outer ends of the text, lines left empty are dropped and the
/// rest are joined with a space.
pub(super) fn handle_white_space(value: &Wtf8) -> Cow<'_, Wtf8> {
    let bytes = value.as_bytes();

    if !bytes.iter().any(|&b| matches!(b, b'\n' | b'\r' | b'\t')) {
        return Cow::Borrowed(value);
    }
    if !bytes.iter().any(|&b| b == b'\n' || b == b'\r') {
        let mut buf = Wtf8Buf::with_capacity(bytes.len());
        push_without_tabs(&mut buf, value);
        return Cow::Owned(buf);
    }
    // The indentation between elements, the most common text by far
    if bytes
        .iter()
        .all(|&b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
    {
        return Cow::Owned(Wtf8Buf::new());
    }

    // Split at \r\n, \n and \r
    let mut lines = vec![];
    let mut line_start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if matches!(bytes[i], b'\n' | b'\r') {
            lines.push(line_start..i);
            if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
                i += 1;
            }
            line_start = i + 1;
        }
        i += 1;
    }
    lines.push(line_start..bytes.len());

    let last_line = lines.len() - 1;
    let last_non_empty_line = (1..=last_line)
        .rev()
        .find(|&i| bytes[lines[i].clone()].iter().any(|&b| !is_blank(b)))
        .unwrap_or(0);
    let mut buf = Wtf8Buf::with_capacity(bytes.len());

    for (i, line) in lines.into_iter().enumerate() {
        let (mut start, mut end) = (line.start, line.end);

        if i != 0 {
            while start < end && is_blank(bytes[start]) {
                start += 1;
            }
        }
        if i != last_line {
            while end > start && is_blank(bytes[end - 1]) {
                end -= 1;
            }
        }
        if end > start {
            push_without_tabs(&mut buf, value.slice(start, end));
            if i != last_non_empty_line {
                buf.push_char(' ');
            }
        }
    }

    Cow::Owned(buf)
}

/// Whitespace as matched by `\s` in JavaScript regular expressions
fn is_js_whitespace(cp: CodePoint) -> bool {
    matches!(
        cp.to_u32(),
        0x09..=0x0d
            | 0x20
            | 0xa0
            | 0x1680
            | 0x2000..=0x200a
            | 0x2028
            | 0x2029
            | 0x202f
            | 0x205f
            | 0x3000
            | 0xfeff
    )
}

/// Collapses a line break followed by whitespace in a JSX attribute string to a
/// single space, like `value.replace(/\n\s+/g, ' ')` in babel-plugin-inferno.
pub(super) fn collapse_attribute_line_breaks(value: &Wtf8) -> Cow<'_, Wtf8> {
    if !value.as_bytes().contains(&b'\n') {
        return Cow::Borrowed(value);
    }

    let mut buf = Wtf8Buf::with_capacity(value.len());
    let mut code_points = value.code_points().peekable();

    while let Some(cp) = code_points.next() {
        if cp.to_u32() == u32::from(b'\n')
            && code_points
                .peek()
                .is_some_and(|&next| is_js_whitespace(next))
        {
            while code_points
                .next_if(|&next| is_js_whitespace(next))
                .is_some()
            {}
            buf.push_char(' ');
        } else {
            buf.push(cp);
        }
    }

    Cow::Owned(buf)
}

/// Applies a text transformation to an atom, and keeps the atom when the text does not change
pub(super) fn map_text(value: Wtf8Atom, f: for<'a> fn(&'a Wtf8) -> Cow<'a, Wtf8>) -> Wtf8Atom {
    if let Cow::Owned(buf) = f(&value) {
        return buf.into();
    }
    value
}
