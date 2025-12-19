import { createVNode, createTextVNode } from "inferno";
/*#__PURE__*/ createVNode(1, "div", null, [
    createTextVNode("Hello"),
    /*#__PURE__*/ createVNode(1, "span"),
    createTextVNode("World")
], 4);
