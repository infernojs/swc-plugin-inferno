import { createVNode, normalizeProps } from "inferno";
/*#__PURE__*/ normalizeProps(/*#__PURE__*/ createVNode(1, "div", "test", null, 1, {
    foo: "bar",
    ...props
}));
