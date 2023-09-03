/*#__PURE__*/ import { createVNode, createTextVNode } from "inferno";
createVNode(1, "div", null, "wow", 16);
/*#__PURE__*/ createVNode(1, "div", null, "w\xf4w", 16);
/*#__PURE__*/ createVNode(1, "div", null, "w & w", 16);
/*#__PURE__*/ createVNode(1, "div", null, "w & w", 16);
/*#__PURE__*/ createVNode(1, "div", null, "w \xa0 w", 16);
/*#__PURE__*/ createVNode(1, "div", null, "this should not parse as unicode: \\u00a0", 16);
/*#__PURE__*/ createVNode(1, "div", null, "this should parse as nbsp: \xa0 ", 16);
/*#__PURE__*/ createVNode(1, "div", null, [
    createTextVNode("this should parse as unicode: "),
    createTextVNode("\u00a0 ")
], 0);
/*#__PURE__*/ createVNode(1, "div", null, "w < w", 16);
