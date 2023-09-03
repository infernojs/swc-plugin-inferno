import { createVNode, normalizeProps } from "inferno";
var foo = null;
var x = /*#__PURE__*/ normalizeProps(createVNode(1, "div", null, null, 1, {
    ...foo
}));
