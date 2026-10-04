//! The flags of `inferno-vnode-flags` 10 that the generated code uses.

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum VNodeFlags {
    ComponentUnknown = 0,
    HtmlElement = 1,
    HasTextChildren = 2,
    HasNonKeyedChildren = 4,
    HasVNodeChildren = 8,
    HasInvalidChildren = 16,
    HasKeyedChildren = 32,
    SvgElement = 64,
    Fragment = 256,
    InputElement = 512,
    TextareaElement = 2048,
    SelectElement = 4096,
    ContentEditable = 131072,
}

/// The shape of the children, which the plugin works out and the deprecated `createVNode` and
/// `createFragment` take
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

impl ChildFlags {
    const ALL: [ChildFlags; 6] = [
        ChildFlags::UnknownChildren,
        ChildFlags::HasInvalidChildren,
        ChildFlags::HasVNodeChildren,
        ChildFlags::HasNonKeyedChildren,
        ChildFlags::HasKeyedChildren,
        ChildFlags::HasTextChildren,
    ];

    /// The child flags that a number stands for, or `None` when it is not a `ChildFlags` value
    pub fn from_value(value: f64) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|&flags| f64::from(flags as u16) == value)
    }

    /// The `VNodeFlags` child bit; unknown children have no bit and are normalized at runtime
    pub fn child_bit(self) -> u32 {
        match self {
            ChildFlags::UnknownChildren => 0,
            ChildFlags::HasInvalidChildren => VNodeFlags::HasInvalidChildren as u32,
            ChildFlags::HasVNodeChildren => VNodeFlags::HasVNodeChildren as u32,
            ChildFlags::HasNonKeyedChildren => VNodeFlags::HasNonKeyedChildren as u32,
            ChildFlags::HasKeyedChildren => VNodeFlags::HasKeyedChildren as u32,
            ChildFlags::HasTextChildren => VNodeFlags::HasTextChildren as u32,
        }
    }
}
