//! Ported from babel-plugin-inferno `tests/attribute-tables.test.js`: Attribute mapping tables
//!
//! babel-plugin-inferno checks every entry of its attribute tables. The tables are copied
//! here from `lib/lowerCaseAttributes.js`, `lib/attrsSVG.js` and `lib/attributeTransforms.js`,
//! and each test reports all entries that fail.

use crate::helpers::*;

/// `lib/lowerCaseAttributes.js`: lowercased on elements
const LOWER_CASE_ATTRIBUTES: &[&str] = &[
    "accessKey",
    "autoComplete",
    "autoCorrect",
    "autoPictureInPicture",
    "autoPlay",
    "autoCapitalize",
    "autoFocus",
    "autoSave",
    "cellPadding",
    "cellSpacing",
    "charSet",
    "classID",
    "codeBase",
    "colSpan",
    "contextMenu",
    "controlsList",
    "crossOrigin",
    "dateTime",
    "encType",
    "enterKeyHint",
    "exportParts",
    "fetchPriority",
    "formAction",
    "formEncType",
    "formMethod",
    "formNoValidate",
    "formTarget",
    "frameBorder",
    "hrefLang",
    "imageSizes",
    "imageSrcSet",
    "inputMode",
    "isMap",
    "itemID",
    "itemProp",
    "itemRef",
    "itemScope",
    "itemType",
    "keyParams",
    "keyType",
    "marginHeight",
    "maxLength",
    "mediaGroup",
    "minLength",
    "noModule",
    "noValidate",
    "popoverTarget",
    "popoverTargetAction",
    "radioGroup",
    "readOnly",
    "referrerPolicy",
    "rowSpan",
    "spellCheck",
    "srcDoc",
    "srcLang",
    "srcSet",
    "tabIndex",
    "useMap",
];

/// `lib/attrsSVG.js`: mapped on elements
const SVG_ATTRIBUTES: &[(&str, &str)] = &[
    ("accentHeight", "accent-height"),
    ("alignmentBaseline", "alignment-baseline"),
    ("arabicForm", "arabic-form"),
    ("baselineShift", "baseline-shift"),
    ("capHeight", "cap-height"),
    ("clipPath", "clip-path"),
    ("clipRule", "clip-rule"),
    ("colorInterpolation", "color-interpolation"),
    ("colorInterpolationFilters", "color-interpolation-filters"),
    ("colorProfile", "color-profile"),
    ("colorRendering", "color-rendering"),
    ("dominantBaseline", "dominant-baseline"),
    ("enableBackground", "enable-background"),
    ("fillOpacity", "fill-opacity"),
    ("fillRule", "fill-rule"),
    ("floodColor", "flood-color"),
    ("floodOpacity", "flood-opacity"),
    ("fontFamily", "font-family"),
    ("fontSize", "font-size"),
    ("fontSizeAdjust", "font-size-adjust"),
    ("fontStretch", "font-stretch"),
    ("fontStyle", "font-style"),
    ("fontVariant", "font-variant"),
    ("fontWeight", "font-weight"),
    ("fontWidth", "font-width"),
    ("glyphName", "glyph-name"),
    ("glyphOrientationHorizontal", "glyph-orientation-horizontal"),
    ("glyphOrientationVertical", "glyph-orientation-vertical"),
    ("horizAdvX", "horiz-adv-x"),
    ("horizOriginX", "horiz-origin-x"),
    ("imageRendering", "image-rendering"),
    ("letterSpacing", "letter-spacing"),
    ("lightingColor", "lighting-color"),
    ("markerEnd", "marker-end"),
    ("markerMid", "marker-mid"),
    ("markerStart", "marker-start"),
    ("markerHeight", "markerHeight"),
    ("maskType", "mask-type"),
    ("overlinePosition", "overline-position"),
    ("overlineThickness", "overline-thickness"),
    ("paintOrder", "paint-order"),
    ("panose1", "panose-1"),
    ("pointerEvents", "pointer-events"),
    ("renderingIntent", "rendering-intent"),
    ("shapeRendering", "shape-rendering"),
    ("stopColor", "stop-color"),
    ("stopOpacity", "stop-opacity"),
    ("strikethroughPosition", "strikethrough-position"),
    ("strikethroughThickness", "strikethrough-thickness"),
    ("strokeDasharray", "stroke-dasharray"),
    ("strokeDashoffset", "stroke-dashoffset"),
    ("strokeLinecap", "stroke-linecap"),
    ("strokeLinejoin", "stroke-linejoin"),
    ("strokeMiterlimit", "stroke-miterlimit"),
    ("strokeOpacity", "stroke-opacity"),
    ("strokeWidth", "stroke-width"),
    ("textDecoration", "text-decoration"),
    ("textOverflow", "text-overflow"),
    ("textRendering", "text-rendering"),
    ("underlinePosition", "underline-position"),
    ("underlineThickness", "underline-thickness"),
    ("unicodeBidi", "unicode-bidi"),
    ("unicodeRange", "unicode-range"),
    ("unitsPerEm", "units-per-em"),
    ("vAlphabetic", "v-alphabetic"),
    ("vHanging", "v-hanging"),
    ("vIdeographic", "v-ideographic"),
    ("vMathematical", "v-mathematical"),
    ("vectorEffect", "vector-effect"),
    ("vertAdvY", "vert-adv-y"),
    ("vertOriginX", "vert-origin-x"),
    ("vertOriginY", "vert-origin-y"),
    ("whiteSpace", "white-space"),
    ("wordSpacing", "word-spacing"),
    ("writingMode", "writing-mode"),
    ("xHeight", "x-height"),
    ("xlinkActuate", "xlink:actuate"),
    ("xlinkArcrole", "xlink:arcrole"),
    ("xlinkHref", "xlink:href"),
    ("xlinkRole", "xlink:role"),
    ("xlinkShow", "xlink:show"),
    ("xlinkTitle", "xlink:title"),
    ("xlinkType", "xlink:type"),
    ("xmlBase", "xml:base"),
    ("xmlnsXlink", "xmlns:xlink"),
    ("xmlLang", "xml:lang"),
    ("xmlSpace", "xml:space"),
];

/// `lib/attributeTransforms.js`: mapped on elements
const ATTRIBUTE_TRANSFORMS: &[(&str, &str)] = &[
    ("acceptCharset", "accept-charset"),
    ("transformOrigin", "transform-origin"),
    ("textAnchor", "text-anchor"),
    ("httpEquiv", "http-equiv"),
    ("htmlFor", "for"),
];

/// Entries of the tables above that swc-plugin-inferno does not map
const NOT_MAPPED: &[&str] = &[
    "fontWidth",
    "maskType",
    "textOverflow",
    "whiteSpace",
    "textAnchor",
];

fn element_props(flags: u32, tag: &str, name: &str) -> String {
    format!("createVNode({flags}, \"{tag}\", null, null, 1, {{\n  \"{name}\": \"v\"\n}});")
}

fn component_props(name: &str) -> String {
    format!("createComponentVNode(2, Foo, {{\n  \"{name}\": \"v\"\n}});")
}

fn assert_mapped_on_elements(flags: u32, tag: &str, table: &[(&str, &str)], mapped: bool) {
    assert_all(
        table
            .iter()
            .filter(|(name, _)| NOT_MAPPED.contains(name) != mapped),
        |(name, to)| {
            assert_transform(
                &format!("<{tag} {name}=\"v\" />"),
                &element_props(flags, tag, to),
            )
        },
    );
}

fn assert_kept_on_components<'a>(names: impl IntoIterator<Item = &'a str>) {
    assert_all(names, |name| {
        assert_transform(&format!("<Foo {name}=\"v\" />"), &component_props(name))
    });
}

/// lowerCaseAttributes
mod lower_case_attributes {
    use super::*;

    #[test]
    fn should_lowercase_on_elements() {
        assert_all(LOWER_CASE_ATTRIBUTES, |name| {
            assert_transform(
                &format!("<div {name}=\"v\" />"),
                &element_props(1, "div", &name.to_lowercase()),
            )
        });
    }

    #[test]
    fn should_keep_on_components() {
        assert_kept_on_components(LOWER_CASE_ATTRIBUTES.iter().copied());
    }
}

/// attrsSVG
mod attrs_svg {
    use super::*;

    #[test]
    fn should_map_on_elements() {
        assert_mapped_on_elements(32, "rect", SVG_ATTRIBUTES, true);
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not map these SVG attributes"]
    fn should_map_on_elements_not_mapped_by_swc_plugin_inferno() {
        assert_mapped_on_elements(32, "rect", SVG_ATTRIBUTES, false);
    }

    #[test]
    fn should_keep_on_components() {
        assert_kept_on_components(SVG_ATTRIBUTES.iter().map(|(name, _)| *name));
    }
}

/// attributeTransforms
mod attribute_transforms {
    use super::*;

    #[test]
    fn should_map_on_elements() {
        assert_mapped_on_elements(1, "div", ATTRIBUTE_TRANSFORMS, true);
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not map these SVG attributes"]
    fn should_map_on_elements_not_mapped_by_swc_plugin_inferno() {
        assert_mapped_on_elements(1, "div", ATTRIBUTE_TRANSFORMS, false);
    }

    #[test]
    fn should_keep_on_components() {
        assert_kept_on_components(ATTRIBUTE_TRANSFORMS.iter().map(|(name, _)| *name));
    }
}
