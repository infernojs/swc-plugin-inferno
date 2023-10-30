/*#__PURE__*/ import { createVNode, normalizeProps } from "inferno";
normalizeProps(createVNode(1, "div", "test", null, 1, {
    foo: "bar",
    ...props
}));
