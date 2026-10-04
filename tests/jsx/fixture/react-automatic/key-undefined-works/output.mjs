import { newVNode, normalizeProps } from "inferno";
const props = {
    foo: true
};
var x = /*#__PURE__*/ normalizeProps(/*#__PURE__*/ newVNode(17, "div", null, null, {
    ...props
}, undefined));
