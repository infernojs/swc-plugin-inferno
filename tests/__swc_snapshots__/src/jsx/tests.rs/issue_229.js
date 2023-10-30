import { createVNode, createTextVNode, createFragment } from "inferno";
const a = /*#__PURE__*/ createFragment([
    createTextVNode("test")
], 4);
const b = /*#__PURE__*/ createVNode(1, "div", null, "test", 16);
