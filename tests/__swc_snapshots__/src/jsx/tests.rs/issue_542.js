import { createVNode, createTextVNode } from "inferno";
let page = /*#__PURE__*/ createVNode(1, "p", null, [
    createTextVNode("Click "),
    /*#__PURE__*/ createVNode(1, "em", null, "New melody", 16),
    createTextVNode(" listen to a randomly generated melody")
], 4);
