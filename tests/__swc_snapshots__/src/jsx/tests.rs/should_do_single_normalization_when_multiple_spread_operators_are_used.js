import { newComponentVNode, normalizeProps } from "inferno";
/*#__PURE__*/ newComponentVNode(0, FooBar, {
    children: [
        /*#__PURE__*/ normalizeProps(/*#__PURE__*/ newComponentVNode(0, BarFoo, {
            ...magics,
            ...foobars,
            ...props
        })),
        /*#__PURE__*/ newComponentVNode(0, NoNormalize)
    ]
});
