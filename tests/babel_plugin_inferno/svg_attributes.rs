//! Ported from babel-plugin-inferno `tests/svg-attributes.test.js`: SVG attributes (MDN reference)

use crate::helpers::*;

/// Attribute names as they appear in the DOM. `class` and `data-*` are tested separately.
/// MDN spells the DOM property referrerPolicy; the attribute is lowercase because it is not in the
/// HTML spec table.
const MDN_ATTRIBUTES: &[&str] = &[
    "accumulate",
    "additive",
    "alignment-baseline",
    "amplitude",
    "attributeName",
    "attributeType",
    "autofocus",
    "azimuth",
    "baseFrequency",
    "baseline-shift",
    "baseProfile",
    "begin",
    "bias",
    "by",
    "calcMode",
    "clip",
    "clip-path",
    "clip-rule",
    "clipPathUnits",
    "color",
    "color-interpolation",
    "color-interpolation-filters",
    "crossorigin",
    "cursor",
    "cx",
    "cy",
    "d",
    "decoding",
    "diffuseConstant",
    "direction",
    "display",
    "divisor",
    "dominant-baseline",
    "download",
    "dur",
    "dx",
    "dy",
    "edgeMode",
    "elevation",
    "end",
    "exponent",
    "fetchpriority",
    "fill",
    "fill-opacity",
    "fill-rule",
    "filter",
    "filterUnits",
    "flood-color",
    "flood-opacity",
    "font-family",
    "font-size",
    "font-size-adjust",
    "font-stretch",
    "font-style",
    "font-variant",
    "font-weight",
    "font-width",
    "fr",
    "from",
    "fx",
    "fy",
    "glyph-orientation-horizontal",
    "glyph-orientation-vertical",
    "gradientTransform",
    "gradientUnits",
    "height",
    "href",
    "hreflang",
    "id",
    "image-rendering",
    "in",
    "in2",
    "intercept",
    "k1",
    "k2",
    "k3",
    "k4",
    "kernelMatrix",
    "kernelUnitLength",
    "keyPoints",
    "keySplines",
    "keyTimes",
    "lang",
    "lengthAdjust",
    "letter-spacing",
    "lighting-color",
    "limitingConeAngle",
    "marker-end",
    "marker-mid",
    "marker-start",
    "markerHeight",
    "markerUnits",
    "markerWidth",
    "mask",
    "mask-type",
    "maskContentUnits",
    "maskUnits",
    "max",
    "media",
    "method",
    "min",
    "mode",
    "numOctaves",
    "offset",
    "onauxclick",
    "onblur",
    "oncuechange",
    "opacity",
    "operator",
    "order",
    "orient",
    "origin",
    "overflow",
    "paint-order",
    "path",
    "pathLength",
    "patternContentUnits",
    "patternTransform",
    "patternUnits",
    "ping",
    "pointer-events",
    "points",
    "pointsAtX",
    "pointsAtY",
    "pointsAtZ",
    "preserveAlpha",
    "preserveAspectRatio",
    "primitiveUnits",
    "r",
    "radius",
    "referrerpolicy",
    "refX",
    "refY",
    "rel",
    "repeatCount",
    "repeatDur",
    "requiredExtensions",
    "requiredFeatures",
    "restart",
    "result",
    "rotate",
    "rx",
    "ry",
    "scale",
    "seed",
    "shape-rendering",
    "side",
    "slope",
    "spacing",
    "specularConstant",
    "specularExponent",
    "spreadMethod",
    "startOffset",
    "stdDeviation",
    "stitchTiles",
    "stop-color",
    "stop-opacity",
    "stroke",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-miterlimit",
    "stroke-opacity",
    "stroke-width",
    "style",
    "surfaceScale",
    "systemLanguage",
    "tabindex",
    "tableValues",
    "target",
    "targetX",
    "targetY",
    "text-anchor",
    "text-decoration",
    "text-overflow",
    "text-rendering",
    "textLength",
    "to",
    "transform",
    "transform-origin",
    "type",
    "unicode-bidi",
    "values",
    "vector-effect",
    "version",
    "viewBox",
    "visibility",
    "white-space",
    "width",
    "word-spacing",
    "writing-mode",
    "x",
    "x1",
    "x2",
    "xChannelSelector",
    "xlink:actuate",
    "xlink:arcrole",
    "xlink:href",
    "xlink:role",
    "xlink:show",
    "xlink:title",
    "xlink:type",
    "xml:lang",
    "xml:space",
    "y",
    "y1",
    "y2",
    "yChannelSelector",
    "z",
    "zoomAndPan",
];

/// Mixed-case names from the HTML spec table. glyphRef and viewTarget are deprecated and no longer
/// listed on MDN.
const MIXED_CASE_ATTRIBUTES: &[&str] = &[
    "attributeName",
    "attributeType",
    "baseFrequency",
    "baseProfile",
    "calcMode",
    "clipPathUnits",
    "diffuseConstant",
    "edgeMode",
    "filterUnits",
    "glyphRef",
    "gradientTransform",
    "gradientUnits",
    "kernelMatrix",
    "kernelUnitLength",
    "keyPoints",
    "keySplines",
    "keyTimes",
    "lengthAdjust",
    "limitingConeAngle",
    "markerHeight",
    "markerUnits",
    "markerWidth",
    "maskContentUnits",
    "maskUnits",
    "numOctaves",
    "pathLength",
    "patternContentUnits",
    "patternTransform",
    "patternUnits",
    "pointsAtX",
    "pointsAtY",
    "pointsAtZ",
    "preserveAlpha",
    "preserveAspectRatio",
    "primitiveUnits",
    "refX",
    "refY",
    "repeatCount",
    "repeatDur",
    "requiredExtensions",
    "requiredFeatures",
    "specularConstant",
    "specularExponent",
    "spreadMethod",
    "startOffset",
    "stdDeviation",
    "stitchTiles",
    "surfaceScale",
    "systemLanguage",
    "tableValues",
    "targetX",
    "targetY",
    "textLength",
    "viewBox",
    "viewTarget",
    "xChannelSelector",
    "yChannelSelector",
    "zoomAndPan",
];

/// React-style camelCase names of lowercase attributes
const LOWERCASE_ALIASES: &[(&str, &str)] = &[
    ("autoFocus", "autofocus"),
    ("crossOrigin", "crossorigin"),
    ("fetchPriority", "fetchpriority"),
    ("hrefLang", "hreflang"),
    ("referrerPolicy", "referrerpolicy"),
    ("tabIndex", "tabindex"),
];

fn camel_case(name: &str) -> String {
    let mut out = String::new();
    let mut chars = name.chars().peekable();

    while let Some(c) = chars.next() {
        match chars.peek() {
            Some(next) if (c == '-' || c == ':') && next.is_ascii_lowercase() => {
                out.push(next.to_ascii_uppercase());
                chars.next();
            }
            _ => out.push(c),
        }
    }
    out
}

fn rect_props(name: &str) -> String {
    format!("createVNode(32, \"rect\", null, null, 1, {{\n  \"{name}\": \"v\"\n}});")
}

fn assert_camel_case_mapped() {
    assert_all(
        MDN_ATTRIBUTES
            .iter()
            .filter(|name| name.contains(['-', ':'])),
        |name| {
            assert_transform(
                &format!("<rect {}=\"v\" />", camel_case(name)),
                &rect_props(name),
            )
        },
    );
}

/// attributes written as in the DOM
mod attributes_written_as_in_the_dom {
    use super::*;

    #[test]
    fn should_pass_class_as_the_class_name_argument() {
        assert_transform(r#"<rect class="v" />"#, r#"createVNode(32, "rect", "v");"#);
    }

    #[test]
    fn should_keep_data_attributes() {
        assert_transform(
            r#"<rect data-foo="v" />"#,
            r#"createVNode(32, "rect", null, null, 1, {
  "data-foo": "v"
});"#,
        );
    }

    #[test]
    fn should_keep_attributes_written_as_in_the_dom() {
        assert_all(
            MDN_ATTRIBUTES
                .iter()
                .filter(|name| !MIXED_CASE_ATTRIBUTES.contains(name)),
            |name| assert_transform(&format!("<rect {name}=\"v\" />"), &rect_props(name)),
        );
    }
}

/// mixed-case attributes from the HTML spec
mod mixed_case_attributes_from_the_html_spec {
    use super::*;

    #[test]
    fn should_keep_length_adjust_in_camel_case_on_svg_text() {
        assert_transform(
            r#"<svg><text lengthAdjust="spacing" /></svg>"#,
            r#"createVNode(32, "svg", null, createVNode(32, "text", null, null, 1, {
  "lengthAdjust": "spacing"
}), 2);"#,
        );
    }

    #[test]
    fn should_keep_x_channel_selector_and_y_channel_selector_in_camel_case_on_fe_displacement_map()
    {
        assert_transform(
            r#"<feDisplacementMap xChannelSelector="R" yChannelSelector="G" />"#,
            r#"createVNode(32, "feDisplacementMap", null, null, 1, {
  "xChannelSelector": "R",
  "yChannelSelector": "G"
});"#,
        );
    }

    #[test]
    fn should_keep_mixed_case_attributes_in_camel_case() {
        assert_all(MIXED_CASE_ATTRIBUTES, |name| {
            assert_transform(&format!("<rect {name}=\"v\" />"), &rect_props(name))
        });
    }
}

/// camelCase names of hyphenated and namespaced attributes
mod camel_case_names_of_hyphenated_and_namespaced_attributes {
    use super::*;

    #[test]
    fn should_map_camel_case_names() {
        assert_camel_case_mapped();
    }
}

/// camelCase names of presentation attributes on their elements
mod camel_case_names_of_presentation_attributes_on_their_elements {
    use super::*;

    #[test]
    fn should_map_mask_type_to_mask_type_on_mask() {
        assert_transform(
            r#"<mask maskType="alpha" />"#,
            r#"createVNode(32, "mask", null, null, 1, {
  "mask-type": "alpha"
});"#,
        );
    }

    #[test]
    fn should_map_text_overflow_to_text_overflow_on_text() {
        assert_transform(
            r#"<text textOverflow="ellipsis" />"#,
            r#"createVNode(32, "text", null, null, 1, {
  "text-overflow": "ellipsis"
});"#,
        );
    }

    #[test]
    fn should_map_white_space_to_white_space_on_text() {
        assert_transform(
            r#"<text whiteSpace="nowrap" />"#,
            r#"createVNode(32, "text", null, null, 1, {
  "white-space": "nowrap"
});"#,
        );
    }

    #[test]
    fn should_map_font_width_to_font_width_on_text() {
        assert_transform(
            r#"<text fontWidth="condensed" />"#,
            r#"createVNode(32, "text", null, null, 1, {
  "font-width": "condensed"
});"#,
        );
    }
}

/// camelCase names of lowercase attributes
mod camel_case_names_of_lowercase_attributes {
    use super::*;

    #[test]
    fn should_map_camel_case_names_to_lowercase() {
        assert_all(LOWERCASE_ALIASES, |(name, to)| {
            assert_transform(&format!("<rect {name}=\"v\" />"), &rect_props(to))
        });
    }
}
