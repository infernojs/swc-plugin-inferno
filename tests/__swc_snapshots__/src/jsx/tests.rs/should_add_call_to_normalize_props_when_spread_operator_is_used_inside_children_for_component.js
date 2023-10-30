/*#__PURE__*/ import { createComponentVNode, normalizeProps } from "inferno";
createComponentVNode(2, FooBar, {
    children: [
        /*#__PURE__*/ normalizeProps(createComponentVNode(2, BarFoo, {
            ...props
        })),
        /*#__PURE__*/ createComponentVNode(2, NoNormalize)
    ]
});
