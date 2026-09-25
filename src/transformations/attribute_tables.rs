//! Attribute tables of babel-plugin-inferno (`lib/attributeTransforms.js`, `lib/lowerCaseAttributes.js`
//! and `lib/attrsSVG.js`). They only apply to elements, never to components.

/// React attribute names that Inferno spells differently
pub fn react_attribute(name: &str) -> Option<&'static str> {
    Some(match name {
        "acceptCharset" => "accept-charset",
        "transformOrigin" => "transform-origin",
        "textAnchor" => "text-anchor",
        "httpEquiv" => "http-equiv",
        "htmlFor" => "for",
        _ => return None,
    })
}

/// React-style camelCase names of lowercase attributes
pub fn is_lowercase_attribute(name: &str) -> bool {
    matches!(
        name,
        "accessKey"
            | "autoComplete"
            | "autoCorrect"
            | "autoPictureInPicture"
            | "autoPlay"
            | "autoCapitalize"
            | "autoFocus"
            | "autoSave"
            | "cellPadding"
            | "cellSpacing"
            | "charSet"
            | "classID"
            | "codeBase"
            | "colSpan"
            | "contextMenu"
            | "controlsList"
            | "crossOrigin"
            | "dateTime"
            | "encType"
            | "enterKeyHint"
            | "exportParts"
            | "fetchPriority"
            | "formAction"
            | "formEncType"
            | "formMethod"
            | "formNoValidate"
            | "formTarget"
            | "frameBorder"
            | "hrefLang"
            | "imageSizes"
            | "imageSrcSet"
            | "inputMode"
            | "isMap"
            | "itemID"
            | "itemProp"
            | "itemRef"
            | "itemScope"
            | "itemType"
            | "keyParams"
            | "keyType"
            | "marginHeight"
            | "maxLength"
            | "mediaGroup"
            | "minLength"
            | "noModule"
            | "noValidate"
            | "popoverTarget"
            | "popoverTargetAction"
            | "radioGroup"
            | "readOnly"
            | "referrerPolicy"
            | "rowSpan"
            | "spellCheck"
            | "srcDoc"
            | "srcLang"
            | "srcSet"
            | "tabIndex"
            | "useMap"
    )
}

/// React-style camelCase names of hyphenated and namespaced SVG attributes
pub fn svg_attribute(name: &str) -> Option<&'static str> {
    Some(match name {
        "accentHeight" => "accent-height",
        "alignmentBaseline" => "alignment-baseline",
        "arabicForm" => "arabic-form",
        "baselineShift" => "baseline-shift",
        "capHeight" => "cap-height",
        "clipPath" => "clip-path",
        "clipRule" => "clip-rule",
        "colorInterpolation" => "color-interpolation",
        "colorInterpolationFilters" => "color-interpolation-filters",
        "colorProfile" => "color-profile",
        "colorRendering" => "color-rendering",
        "dominantBaseline" => "dominant-baseline",
        "enableBackground" => "enable-background",
        "fillOpacity" => "fill-opacity",
        "fillRule" => "fill-rule",
        "floodColor" => "flood-color",
        "floodOpacity" => "flood-opacity",
        "fontFamily" => "font-family",
        "fontSize" => "font-size",
        "fontSizeAdjust" => "font-size-adjust",
        "fontStretch" => "font-stretch",
        "fontStyle" => "font-style",
        "fontVariant" => "font-variant",
        "fontWeight" => "font-weight",
        "fontWidth" => "font-width",
        "glyphName" => "glyph-name",
        "glyphOrientationHorizontal" => "glyph-orientation-horizontal",
        "glyphOrientationVertical" => "glyph-orientation-vertical",
        "horizAdvX" => "horiz-adv-x",
        "horizOriginX" => "horiz-origin-x",
        "imageRendering" => "image-rendering",
        "letterSpacing" => "letter-spacing",
        "lightingColor" => "lighting-color",
        "markerEnd" => "marker-end",
        "markerMid" => "marker-mid",
        "markerStart" => "marker-start",
        "maskType" => "mask-type",
        "overlinePosition" => "overline-position",
        "overlineThickness" => "overline-thickness",
        "paintOrder" => "paint-order",
        "panose1" => "panose-1",
        "pointerEvents" => "pointer-events",
        "renderingIntent" => "rendering-intent",
        "shapeRendering" => "shape-rendering",
        "stopColor" => "stop-color",
        "stopOpacity" => "stop-opacity",
        "strikethroughPosition" => "strikethrough-position",
        "strikethroughThickness" => "strikethrough-thickness",
        "strokeDasharray" => "stroke-dasharray",
        "strokeDashoffset" => "stroke-dashoffset",
        "strokeLinecap" => "stroke-linecap",
        "strokeLinejoin" => "stroke-linejoin",
        "strokeMiterlimit" => "stroke-miterlimit",
        "strokeOpacity" => "stroke-opacity",
        "strokeWidth" => "stroke-width",
        "textDecoration" => "text-decoration",
        "textOverflow" => "text-overflow",
        "textRendering" => "text-rendering",
        "underlinePosition" => "underline-position",
        "underlineThickness" => "underline-thickness",
        "unicodeBidi" => "unicode-bidi",
        "unicodeRange" => "unicode-range",
        "unitsPerEm" => "units-per-em",
        "vAlphabetic" => "v-alphabetic",
        "vHanging" => "v-hanging",
        "vIdeographic" => "v-ideographic",
        "vMathematical" => "v-mathematical",
        "vectorEffect" => "vector-effect",
        "vertAdvY" => "vert-adv-y",
        "vertOriginX" => "vert-origin-x",
        "vertOriginY" => "vert-origin-y",
        "whiteSpace" => "white-space",
        "wordSpacing" => "word-spacing",
        "writingMode" => "writing-mode",
        "xHeight" => "x-height",
        "xlinkActuate" => "xlink:actuate",
        "xlinkArcrole" => "xlink:arcrole",
        "xlinkHref" => "xlink:href",
        "xlinkRole" => "xlink:role",
        "xlinkShow" => "xlink:show",
        "xlinkTitle" => "xlink:title",
        "xlinkType" => "xlink:type",
        "xmlBase" => "xml:base",
        "xmlnsXlink" => "xmlns:xlink",
        "xmlLang" => "xml:lang",
        "xmlSpace" => "xml:space",
        _ => return None,
    })
}
