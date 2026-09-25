import { createVNode, createTextVNode } from "inferno";
var x = /*#__PURE__*/ createVNode(1, "div", null, [
    /*#__PURE__*/ createTextVNode("foo"),
    /*#__PURE__*/ createTextVNode("bar"),
    /*#__PURE__*/ createTextVNode("baz"),
    /*#__PURE__*/ createVNode(1, "div", null, "buz bang", 16),
    /*#__PURE__*/ createTextVNode("qux"),
    null,
    /*#__PURE__*/ createTextVNode("quack")
], 0);
