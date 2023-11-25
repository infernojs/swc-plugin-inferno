/*#__PURE__*/ import { createVNode, createTextVNode, createFragment } from "inferno";
createFragment([
    /*#__PURE__*/ createVNode(1, "span", null, "Hello something long to not trigger line break", 16),
    /*#__PURE__*/ createTextVNode(" ")
], 4);
