import { createVNode, normalizeProps } from "inferno";
const props = {
    foo: true
};
var x = /*#__PURE__*/ normalizeProps(/*#__PURE__*/ createVNode(1, "div", null, null, 1, {
    ...props
}, undefined));
