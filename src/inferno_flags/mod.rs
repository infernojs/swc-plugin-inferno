//! The flags of `inferno-vnode-flags` that the generated code uses.

#[repr(u16)]
#[derive(Copy, Clone)]
pub enum VNodeFlags {
    HtmlElement = 1,
    ComponentUnknown = 2,
    SvgElement = 32,
    InputElement = 64,
    TextareaElement = 128,
    SelectElement = 256,
    ReCreate = 2048,
    ContentEditable = 4096,
}

#[expect(clippy::enum_variant_names)]
#[repr(u16)]
#[derive(Copy, Clone)]
pub enum ChildFlags {
    UnknownChildren = 0,
    HasInvalidChildren = 1,
    HasVNodeChildren = 2,
    HasNonKeyedChildren = 4,
    HasKeyedChildren = 8,
    HasTextChildren = 16,
}
