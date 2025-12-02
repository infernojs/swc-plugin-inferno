import { createComponentVNode, normalizeProps } from "inferno";
/*#__PURE__*/ createComponentVNode(2, FooBar, {
    children: [
        /*#__PURE__*/ normalizeProps(createComponentVNode(2, BarFoo, {
            ...magics,
            ...foobars,
            ...props
        })),
        /*#__PURE__*/ createComponentVNode(2, NoNormalize)
    ]
});
