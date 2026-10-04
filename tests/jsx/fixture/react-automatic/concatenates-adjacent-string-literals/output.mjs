import { newVNode, newTextVNode } from "inferno";
var x = /*#__PURE__*/ newVNode(1, "div", null, [
    /*#__PURE__*/ newTextVNode("foo"),
    /*#__PURE__*/ newTextVNode("bar"),
    /*#__PURE__*/ newTextVNode("baz"),
    /*#__PURE__*/ newVNode(3, "div", null, "buz bang"),
    /*#__PURE__*/ newTextVNode("qux"),
    null,
    /*#__PURE__*/ newTextVNode("quack")
]);
