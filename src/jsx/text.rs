use swc_core::atoms::wtf8::{CodePoint, Wtf8, Wtf8Buf};

fn is(cp: CodePoint, c: char) -> bool {
    cp.to_u32() == c as u32
}

fn is_blank(cp: CodePoint) -> bool {
    is(cp, ' ') || is(cp, '\t')
}

fn push_without_tabs(buf: &mut Wtf8Buf, code_points: &[CodePoint]) {
    for &cp in code_points {
        if is(cp, '\t') {
            buf.push_char(' ');
        } else {
            buf.push(cp);
        }
    }
}

/// Collapses the whitespace of JSX text like `handleWhiteSpace` of
/// babel-plugin-inferno: tabs become spaces, the lines are trimmed of spaces
/// except at the outer ends of the text, lines left empty are dropped and the
/// rest are joined with a space.
pub(super) fn handle_white_space(value: &Wtf8) -> Wtf8Buf {
    let code_points: Vec<CodePoint> = value.code_points().collect();
    let mut buf = Wtf8Buf::with_capacity(value.len());

    if !code_points.iter().any(|&cp| is(cp, '\n') || is(cp, '\r')) {
        push_without_tabs(&mut buf, &code_points);
        return buf;
    }
    // The indentation between elements, the most common text by far
    if code_points
        .iter()
        .all(|&cp| is_blank(cp) || is(cp, '\r') || is(cp, '\n'))
    {
        return buf;
    }

    // Split at \r\n, \n and \r
    let mut lines: Vec<&[CodePoint]> = vec![];
    let mut line_start = 0;
    let mut i = 0;
    while i < code_points.len() {
        let cp = code_points[i];
        if is(cp, '\n') || is(cp, '\r') {
            lines.push(&code_points[line_start..i]);
            if is(cp, '\r') && code_points.get(i + 1).is_some_and(|&next| is(next, '\n')) {
                i += 1;
            }
            line_start = i + 1;
        }
        i += 1;
    }
    lines.push(&code_points[line_start..]);

    let last_line = lines.len() - 1;
    let last_non_empty_line = (1..=last_line)
        .rev()
        .find(|&i| lines[i].iter().any(|&cp| !is_blank(cp)))
        .unwrap_or(0);

    for (i, line) in lines.iter().enumerate() {
        let mut start = 0;
        let mut end = line.len();

        if i != 0 {
            while start < end && is_blank(line[start]) {
                start += 1;
            }
        }
        if i != last_line {
            while end > start && is_blank(line[end - 1]) {
                end -= 1;
            }
        }
        if end > start {
            push_without_tabs(&mut buf, &line[start..end]);
            if i != last_non_empty_line {
                buf.push_char(' ');
            }
        }
    }

    buf
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
pub(super) fn collapse_attribute_line_breaks(value: &Wtf8) -> Wtf8Buf {
    let mut buf = Wtf8Buf::with_capacity(value.len());
    let mut code_points = value.code_points().peekable();

    while let Some(cp) = code_points.next() {
        if is(cp, '\n')
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

    buf
}
