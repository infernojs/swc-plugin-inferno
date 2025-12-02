import { createVNode, createTextVNode } from "inferno";
/*#__PURE__*/ createVNode(1, "div", null, "wow", 16);
/*#__PURE__*/ createVNode(1, "div", null, "wôw", 16);
/*#__PURE__*/ createVNode(1, "div", null, "w & w", 16);
/*#__PURE__*/ createVNode(1, "div", null, "w & w", 16);
/*#__PURE__*/ createVNode(1, "div", null, "w   w", 16);
/*#__PURE__*/ createVNode(1, "div", null, [
    createTextVNode("this should parse as unicode: "),
    createTextVNode('\u00a0 ')
], 0);
/*#__PURE__*/ createVNode(1, "div", null, "w < w", 16);
