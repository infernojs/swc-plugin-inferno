import { createVNode, createTextVNode } from "inferno";
var x = /*#__PURE__*/ createVNode(1, "div", null, [
    createTextVNode("foo"),
    createTextVNode("bar"),
    createTextVNode("baz"),
    /*#__PURE__*/ createVNode(1, "div", null, "buz bang", 16),
    createTextVNode("qux"),
    null,
    createTextVNode("quack")
], 0);
