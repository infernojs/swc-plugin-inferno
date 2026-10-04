import { newVNode, normalizeProps } from "inferno";
import * as inferno from "inferno";
var y = /*#__PURE__*/ inferno.createElement("div", {
    foo: 1
});
var x = /*#__PURE__*/ newVNode(33, "div", null, [
    /*#__PURE__*/ newVNode(17, "div", null, null, null, "1"),
    /*#__PURE__*/ newVNode(17, "div", null, null, {
        meow: "wolf"
    }, "2"),
    /*#__PURE__*/ newVNode(17, "div", null, null, null, "3"),
    /*#__PURE__*/ normalizeProps(/*#__PURE__*/ newVNode(17, "div", null, null, {
        ...props
    }, "4"))
]);
