use swc_core::{
    ecma::atoms::{js_word, JsWord},
    ecma::ast::{Ident, PropName, Str},

};
use crate::atoms;

use crate::inferno_flags::VNodeFlags;

pub fn parse_vnode_flag(tag: &JsWord) -> u16 {
    match *tag {
        js_word!("input") => VNodeFlags::InputElement as u16,
        js_word!("textarea") => VNodeFlags::TextareaElement as u16,
        js_word!("select") => VNodeFlags::SelectElement as u16,
        // SVG ELEMENTS
        js_word!("altGlyph") => VNodeFlags::SvgElement as u16,
        js_word!("altGlyphDef") => VNodeFlags::SvgElement as u16,
        js_word!("altGlyphItem") => VNodeFlags::SvgElement as u16,
        js_word!("animate") => VNodeFlags::SvgElement as u16,
        js_word!("animateColor") => VNodeFlags::SvgElement as u16,
        js_word!("animateMotion") => VNodeFlags::SvgElement as u16,
        js_word!("animateTransform") => VNodeFlags::SvgElement as u16,
        js_word!("circle") => VNodeFlags::SvgElement as u16,
        js_word!("clipPath") => VNodeFlags::SvgElement as u16,
        js_word!("color-profile") => VNodeFlags::SvgElement as u16,
        js_word!("cursor") => VNodeFlags::SvgElement as u16,
        js_word!("defs") => VNodeFlags::SvgElement as u16,
        js_word!("desc") => VNodeFlags::SvgElement as u16,
        js_word!("discard") => VNodeFlags::SvgElement as u16,
        js_word!("ellipse") => VNodeFlags::SvgElement as u16,
        js_word!("feBlend") => VNodeFlags::SvgElement as u16,
        js_word!("feColorMatrix") => VNodeFlags::SvgElement as u16,
        js_word!("feComponentTransfer") => VNodeFlags::SvgElement as u16,
        js_word!("feComposite") => VNodeFlags::SvgElement as u16,
        js_word!("feConvolveMatrix") => VNodeFlags::SvgElement as u16,
        js_word!("feDiffuseLighting") => VNodeFlags::SvgElement as u16,
        js_word!("feDisplacementMap") => VNodeFlags::SvgElement as u16,
        js_word!("feDistantLight") => VNodeFlags::SvgElement as u16,
        js_word!("feDropShadow") => VNodeFlags::SvgElement as u16,
        js_word!("feFlood") => VNodeFlags::SvgElement as u16,
        js_word!("feFuncA") => VNodeFlags::SvgElement as u16,
        js_word!("feFuncB") => VNodeFlags::SvgElement as u16,
        js_word!("feFuncG") => VNodeFlags::SvgElement as u16,
        js_word!("feFuncR") => VNodeFlags::SvgElement as u16,
        js_word!("feGaussianBlur") => VNodeFlags::SvgElement as u16,
        js_word!("feImage") => VNodeFlags::SvgElement as u16,
        js_word!("feMerge") => VNodeFlags::SvgElement as u16,
        js_word!("feMergeNode") => VNodeFlags::SvgElement as u16,
        js_word!("feMorphology") => VNodeFlags::SvgElement as u16,
        js_word!("feOffset") => VNodeFlags::SvgElement as u16,
        js_word!("fePointLight") => VNodeFlags::SvgElement as u16,
        js_word!("feSpecularLighting") => VNodeFlags::SvgElement as u16,
        js_word!("feSpotLight") => VNodeFlags::SvgElement as u16,
        js_word!("feTile") => VNodeFlags::SvgElement as u16,
        js_word!("feTurbulence") => VNodeFlags::SvgElement as u16,
        js_word!("filter") => VNodeFlags::SvgElement as u16,
        js_word!("font-face") => VNodeFlags::SvgElement as u16,
        js_word!("font-face-format") => VNodeFlags::SvgElement as u16,
        js_word!("font-face-name") => VNodeFlags::SvgElement as u16,
        js_word!("font-face-src") => VNodeFlags::SvgElement as u16,
        js_word!("font-face-uri") => VNodeFlags::SvgElement as u16,
        js_word!("foreignObject") => VNodeFlags::SvgElement as u16,
        js_word!("g") => VNodeFlags::SvgElement as u16,
        js_word!("glyph") => VNodeFlags::SvgElement as u16,
        js_word!("glyphRef") => VNodeFlags::SvgElement as u16,
        js_word!("hkern") => VNodeFlags::SvgElement as u16,
        js_word!("line") => VNodeFlags::SvgElement as u16,
        js_word!("linearGradient") => VNodeFlags::SvgElement as u16,
        js_word!("marker") => VNodeFlags::SvgElement as u16,
        js_word!("mask") => VNodeFlags::SvgElement as u16,
        js_word!("metadata") => VNodeFlags::SvgElement as u16,
        js_word!("missing-glyph") => VNodeFlags::SvgElement as u16,
        js_word!("mpath") => VNodeFlags::SvgElement as u16,
        js_word!("path") => VNodeFlags::SvgElement as u16,
        js_word!("pattern") => VNodeFlags::SvgElement as u16,
        js_word!("polygon") => VNodeFlags::SvgElement as u16,
        js_word!("polyline") => VNodeFlags::SvgElement as u16,
        js_word!("radialGradient") => VNodeFlags::SvgElement as u16,
        js_word!("rect") => VNodeFlags::SvgElement as u16,
        js_word!("set") => VNodeFlags::SvgElement as u16,
        js_word!("stop") => VNodeFlags::SvgElement as u16,
        js_word!("svg") => VNodeFlags::SvgElement as u16,
        js_word!("switch") => VNodeFlags::SvgElement as u16,
        js_word!("symbol") => VNodeFlags::SvgElement as u16,
        js_word!("text") => VNodeFlags::SvgElement as u16,
        js_word!("textPath") => VNodeFlags::SvgElement as u16,
        js_word!("tref") => VNodeFlags::SvgElement as u16,
        js_word!("tspan") => VNodeFlags::SvgElement as u16,
        js_word!("unknown") => VNodeFlags::SvgElement as u16,
        js_word!("use") => VNodeFlags::SvgElement as u16,
        js_word!("view") => VNodeFlags::SvgElement as u16,
        js_word!("vkern") => VNodeFlags::SvgElement as u16,

        _ => {
            if tag == &*atoms::ATOM_HATCH {
                return VNodeFlags::SvgElement as u16;
            }
            if tag == &*atoms::ATOM_HATCH_PATH {
                return VNodeFlags::SvgElement as u16;
            }
            if tag == &*atoms::ATOM_MESH {
                return VNodeFlags::SvgElement as u16;
            }
            if tag == &*atoms::ATOM_MESH_GRADIENT {
                return VNodeFlags::SvgElement as u16;
            }
            if tag == &*atoms::ATOM_MESH_PATCH {
                return VNodeFlags::SvgElement as u16;
            }
            if tag == &*atoms::ATOM_MESH_ROW {
                return VNodeFlags::SvgElement as u16;
            }
            if tag == &*atoms::ATOM_SOLID_COLOR {
                return VNodeFlags::SvgElement as u16;
            }

            return VNodeFlags::HtmlElement as u16;
        }
    }
}

pub fn convert_svg_attrs(prop_ident: Ident) -> PropName {
    let new_name;

    if prop_ident.sym == *atoms::ATOM_ACCENTHEIGHT {
        new_name = atoms::ATOM_ACCENT_HEIGHT.clone();
    } else if prop_ident.sym == *atoms::ATOM_ALIGNMENTBASELINE {
        new_name = atoms::ATOM_ALIGNMENT_BASELINE.clone();
    }
    else if prop_ident.sym == js_word!("clipPath") {
        new_name = js_word!("clip-path")
    }
    else if prop_ident.sym == *atoms::ATOM_COLORPROFILE {
        new_name = js_word!("color-profile")
    }
    else if prop_ident.sym == *atoms::ATOM_FILLOPACITY {
        new_name = js_word!("fill-opacity")
    }
    else if prop_ident.sym == *atoms::ATOM_FONTSIZE {
        new_name = js_word!("font-size")
    }
    else if prop_ident.sym == *atoms::ATOM_FONTSIZEADJUST {
        new_name = js_word!("font-size-adjust")
    }
    else if prop_ident.sym == *atoms::ATOM_FONTSTRETCH {
        new_name = js_word!("font-stretch")
    }
    else if prop_ident.sym == *atoms::ATOM_FONTSTYLE {
        new_name = js_word!("font-style")
    }
    else if prop_ident.sym == *atoms::ATOM_FONTVARIANT {
        new_name = js_word!("font-variant")
    }
    else if prop_ident.sym == *atoms::ATOM_FONTWEIGHT {
        new_name = js_word!("font-weight")
    }
    else if prop_ident.sym == *atoms::ATOM_IMAGERENDERING {
        new_name = js_word!("image-rendering")
    }
    else if prop_ident.sym == *atoms::ATOM_LETTERSPACING {
        new_name = js_word!("letter-spacing")
    }
    else if prop_ident.sym == *atoms::ATOM_PAINTORDER {
        new_name = js_word!("paint-order")
    }
    else if prop_ident.sym == *atoms::ATOM_PANOSE1 {
        new_name = js_word!("panose-1")
    }
    else if prop_ident.sym == *atoms::ATOM_POINTEREVENTS {
        new_name = js_word!("pointer-events")
    }
    else if prop_ident.sym == *atoms::ATOM_STROKEDASHARRAY {
        new_name = js_word!("stroke-dasharray")
    }
    else if prop_ident.sym == *atoms::ATOM_STROKEOPACITY {
        new_name = js_word!("stroke-opacity")
    }
    else if prop_ident.sym == *atoms::ATOM_TEXTDECORATION {
        new_name = js_word!("text-decoration")
    }
    else if prop_ident.sym == *atoms::ATOM_TEXTRENDERING {
        new_name = js_word!("text-rendering")
    }
    else if prop_ident.sym == *atoms::ATOM_UNICODEBIDI {
        new_name = js_word!("unicode-bidi")
    }
    else if prop_ident.sym == *atoms::ATOM_WORDSPACING {
        new_name = js_word!("word-spacing")
    }
    else if prop_ident.sym == *atoms::ATOM_WRITINGMODE {
        new_name = js_word!("writing-mode")
    }
    else if prop_ident.sym == *atoms::ATOM_XLINKACTUATE {
        new_name = js_word!("xlink:actuate")
    }
    else if prop_ident.sym == *atoms::ATOM_XLINKARCROLE {
        new_name = js_word!("xlink:arcrole")
    }
    else if prop_ident.sym == *atoms::ATOM_XLINKHREF {
        new_name = js_word!("xlink:href")
    }
    else if prop_ident.sym == *atoms::ATOM_XLINKROLE {
        new_name = js_word!("xlink:role")
    }
    else if prop_ident.sym == *atoms::ATOM_XLINKSHOW {
        new_name = js_word!("xlink:show")
    }
    else if prop_ident.sym == *atoms::ATOM_XLINKTITLE {
        new_name = js_word!("xlink:title")
    }
    else if prop_ident.sym == *atoms::ATOM_XLINKTYPE {
        new_name = js_word!("xlink:type")
    }
    else if prop_ident.sym == *atoms::ATOM_XMLNSXLINK {
        new_name = js_word!("xmlns:xlink")
    }
    else if prop_ident.sym == *atoms::ATOM_XMLLANG {
        new_name = js_word!("xml:lang")
    }
    else if prop_ident.sym == *atoms::ATOM_XMLSPACE {
        new_name = js_word!("xml:space")
    }
    else if prop_ident.sym == *atoms::ATOM_ALIGNMENTBASELINE {
        new_name = atoms::ATOM_ALIGNMENT_BASELINE.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_ARABICFORM {
        new_name = atoms::ATOM_ARABIC_FORM.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_BASELINESHIFT {
        new_name = atoms::ATOM_BASELINE_SHIFT.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_CAPHEIGHT {
        new_name = atoms::ATOM_CAP_HEIGHT.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_CLIPRULE {
        new_name = atoms::ATOM_CLIP_RULE.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_COLORINTERPOLATION {
        new_name = atoms::ATOM_COLOR_INTERPOLATION.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_COLORINTERPOLATIONFILTERS {
        new_name = atoms::ATOM_COLOR_INTERPOLATION_FILTERS.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_COLORRENDERING {
        new_name = atoms::ATOM_COLOR_RENDERING.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_DOMINANTBASELINE {
        new_name = atoms::ATOM_DOMINANT_BASELINE.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_ENABLEBACKGROUND {
        new_name = atoms::ATOM_ENABLE_BACKGROUND.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_FILLRULE {
        new_name = atoms::ATOM_FILL_RULE.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_FLOODCOLOR {
        new_name = atoms::ATOM_FLOOD_COLOR.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_FLOODOPACITY {
        new_name = atoms::ATOM_FLOOD_OPACITY.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_FONTFAMILY {
        new_name = atoms::ATOM_FONT_FAMILY.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_GLYPHNAME {
        new_name = atoms::ATOM_GLYPH_NAME.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_GLYPHORIENTATIONHORIZONTAL {
        new_name = atoms::ATOM_GLYPH_ORIENTATION_HORIZONTAL.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_GLYPHORIENTATIONVERTICAL {
        new_name = atoms::ATOM_GLYPH_ORIENTATION_VERTICAL.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_HORIZADVX {
        new_name = atoms::ATOM_HORIZ_ADV_X.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_HORIZORIGINX {
        new_name = atoms::ATOM_HORIZ_ORIGIN_X.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_LIGHTINGCOLOR {
        new_name = atoms::ATOM_LIGHTING_COLOR.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_MARKEREND {
        new_name = atoms::ATOM_MARKER_END.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_MARKERMID {
        new_name = atoms::ATOM_MARKER_MID.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_MARKERSTART {
        new_name = atoms::ATOM_MARKER_START.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_OVERLINEPOSITION {
        new_name = atoms::ATOM_OVERLINE_POSITION.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_OVERLINETHICKNESS {
        new_name = atoms::ATOM_OVERLINE_THICKNESS.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_RENDERINGINTENT {
        new_name = atoms::ATOM_RENDERING_INTENT.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_SHAPERENDERING {
        new_name = atoms::ATOM_SHAPE_RENDERING.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_STOPCOLOR {
        new_name = atoms::ATOM_STOP_COLOR.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_STOPOPACITY {
        new_name = atoms::ATOM_STOP_OPACITY.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_STRIKETHROUGHPOSITION {
        new_name = atoms::ATOM_STRIKETHROUGH_POSITION.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_STRIKETHROUGHTHICKNESS {
        new_name = atoms::ATOM_STRIKETHROUGH_THICKNESS.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_UNDERLINEPOSITION {
        new_name = atoms::ATOM_UNDERLINE_POSITION.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_UNDERLINETHICKNESS {
        new_name = atoms::ATOM_UNDERLINE_THICKNESS.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_STROKEDASHOFFSET {
        new_name = atoms::ATOM_STROKE_DASHOFFSET.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_STROKELINECAP {
        new_name = atoms::ATOM_STROKE_LINECAP.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_STROKELINEJOIN {
        new_name = atoms::ATOM_STROKE_LINEJOIN.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_STROKEMITERLIMIT {
        new_name = atoms::ATOM_STROKE_MITERLIMIT.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_STROKEWIDTH {
        new_name = atoms::ATOM_STROKE_WIDTH.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_UNICODERANGE {
        new_name = atoms::ATOM_UNICODE_RANGE.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_UNITSPEREM {
        new_name = atoms::ATOM_UNITS_PER_EM.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_VALPHABETIC {
        new_name = atoms::ATOM_V_ALPHABETIC.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_VHANGING {
        new_name = atoms::ATOM_V_HANGING.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_VIDEOGRAPHIC {
        new_name = atoms::ATOM_V_IDEOGRAPHIC.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_VMATHEMATICAL {
        new_name = atoms::ATOM_V_MATHEMATICAL.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_VECTOREFFECT {
        new_name = atoms::ATOM_VECTOR_EFFECT.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_VERTADVY {
        new_name = atoms::ATOM_VERT_ADV_Y.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_VERTORIGINX {
        new_name = atoms::ATOM_VERT_ORIGIN_X.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_VERTORIGINY {
        new_name = atoms::ATOM_VERT_ORIGIN_Y.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_XHEIGHT {
        new_name = atoms::ATOM_X_HEIGHT.clone()
    }
    else if prop_ident.sym == *atoms::ATOM_XMLBASE {
        new_name = atoms::ATOM_XML_BASE.clone()
    } else {
        if prop_ident.sym.contains('-') {
            return PropName::Str(Str {
                span: prop_ident.span,
                raw: None,
                value: prop_ident.sym,
            });
        } else {
            return PropName::Ident(prop_ident);
        }
    }

    return PropName::Str(Str {
        span: prop_ident.span,
        raw: None,
        value: new_name,
    });
}
