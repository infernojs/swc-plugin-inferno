import { newVNode, newTextVNode } from "inferno";
/*#__PURE__*/ newVNode(3, "div", null, "wow");
/*#__PURE__*/ newVNode(3, "div", null, "wôw");
/*#__PURE__*/ newVNode(3, "div", null, "w & w");
/*#__PURE__*/ newVNode(3, "div", null, "w & w");
/*#__PURE__*/ newVNode(3, "div", null, "w   w");
/*#__PURE__*/ newVNode(3, "div", null, "this should not parse as unicode: \\u00a0");
/*#__PURE__*/ newVNode(3, "div", null, "this should parse as nbsp:   ");
/*#__PURE__*/ newVNode(1, "div", null, [
    /*#__PURE__*/ newTextVNode("this should parse as unicode: "),
    /*#__PURE__*/ newTextVNode("\u00a0 ")
]);
/*#__PURE__*/ newVNode(3, "div", null, "w < w");
