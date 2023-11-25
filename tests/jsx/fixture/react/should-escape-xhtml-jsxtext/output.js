/*#__PURE__*/ import { createVNode, createTextVNode } from "inferno";
createVNode(1, "div", null, "wow", 16);
/*#__PURE__*/ createVNode(1, "div", null, "wôw", 16);
/*#__PURE__*/ createVNode(1, "div", null, "w & w", 16);
/*#__PURE__*/ createVNode(1, "div", null, "w & w", 16);
/*#__PURE__*/ createVNode(1, "div", null, "w   w", 16);
/*#__PURE__*/ createVNode(1, "div", null, "this should not parse as unicode: \\u00a0", 16);
/*#__PURE__*/ createVNode(1, "div", null, "this should parse as nbsp:   ", 16);
/*#__PURE__*/ createVNode(1, "div", null, [
    /*#__PURE__*/ createTextVNode("this should parse as unicode: "),
    /*#__PURE__*/ createTextVNode("\u00a0 ")
], 0);
/*#__PURE__*/ createVNode(1, "div", null, "w < w", 16);
